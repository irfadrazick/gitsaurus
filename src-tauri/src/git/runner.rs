use super::env::git_env;
use super::error::{GitError, GitResult};
use std::io::{ErrorKind, Read, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// A command that talks to a remote (fetch, pull, push) is stopped after this long with no
/// output at all. There is no overall deadline: `--progress` keeps a working transfer
/// reporting, so only a stalled connection (or a hook silent for this long) reaches it.
pub const NETWORK_STALL_LIMIT: Duration = Duration::from_secs(300);

/// After SIGTERM, how long git gets to remove its lock files before SIGKILL.
const TERM_GRACE: Duration = Duration::from_secs(2);

#[derive(Debug, Clone)]
pub struct Git {
    repo: PathBuf,
}

impl Git {
    pub fn new(repo: &Path) -> Git {
        Git {
            repo: repo.to_path_buf(),
        }
    }

    pub fn repo(&self) -> &Path {
        &self.repo
    }

    pub fn run(&self, args: &[&str]) -> GitResult<String> {
        self.exec(args, &[0], None, &[], None)
    }

    /// Like `run`, but once the child has produced no output for `limit` it (and anything it
    /// spawned, e.g. ssh) is stopped, returning `Failed` "No progress for N, stopped".
    pub fn run_with_stall_limit(&self, args: &[&str], limit: Duration) -> GitResult<String> {
        self.exec(args, &[0], None, &[], Some(limit))
    }

    /// A command that talks to a remote. `args[0]` is the subcommand; `--progress` is added
    /// after it so a slow but working transfer keeps resetting the stall limit.
    pub fn run_network(&self, args: &[&str]) -> GitResult<String> {
        let mut with_progress = Vec::with_capacity(args.len() + 1);
        with_progress.extend(args.first());
        with_progress.push("--progress");
        with_progress.extend(args.iter().skip(1));
        self.run_with_stall_limit(&with_progress, NETWORK_STALL_LIMIT)
    }

    /// Read-only command: avoids taking optional index locks.
    pub fn run_read(&self, args: &[&str]) -> GitResult<String> {
        self.exec(args, &[0], None, &[("GIT_OPTIONAL_LOCKS", "0")], None)
    }

    /// Read-only command that tolerates the given non-zero exit codes.
    pub fn run_read_ok_codes(&self, args: &[&str], ok: &[i32]) -> GitResult<String> {
        self.exec(args, ok, None, &[("GIT_OPTIONAL_LOCKS", "0")], None)
    }

    pub fn run_ok_codes(&self, args: &[&str], ok: &[i32]) -> GitResult<String> {
        self.exec(args, ok, None, &[], None)
    }

    pub fn run_stdin(&self, args: &[&str], stdin: &str) -> GitResult<String> {
        self.exec(args, &[0], Some(stdin), &[], None)
    }

    pub fn run_env(&self, args: &[&str], extra_env: &[(&str, &str)]) -> GitResult<String> {
        self.exec(args, &[0], None, extra_env, None)
    }

    /// Absolute path of a file inside the git dir (`rev-parse --git-path`).
    pub fn git_path(&self, name: &str) -> GitResult<PathBuf> {
        let out = self.run_read(&["rev-parse", "--git-path", name])?;
        let p = PathBuf::from(out.trim_end_matches('\n'));
        Ok(if p.is_absolute() {
            p
        } else {
            self.repo.join(p)
        })
    }

    fn exec(
        &self,
        args: &[&str],
        ok: &[i32],
        stdin: Option<&str>,
        extra_env: &[(&str, &str)],
        stall_limit: Option<Duration>,
    ) -> GitResult<String> {
        if !self.repo.is_dir() {
            return Err(GitError::NotARepo {
                message: format!("Not a git repository: {}", self.repo.display()),
            });
        }
        let mut cmd = Command::new("git");
        cmd.arg("-C")
            .arg(&self.repo)
            .args(args)
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("LC_ALL", "C")
            .env("GIT_LITERAL_PATHSPECS", "1")
            .stdin(if stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (k, v) in git_env() {
            cmd.env(k, v);
        }
        for (k, v) in extra_env {
            cmd.env(k, v);
        }
        if stall_limit.is_some() {
            // Own process group, so a stall stops git's helpers (ssh, remote-https) too.
            cmd.process_group(0);
        }
        let mut child = cmd.spawn().map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                GitError::GitMissing {
                    message: "git executable not found".into(),
                }
            } else {
                GitError::Failed {
                    message: e.to_string(),
                    stderr: String::new(),
                }
            }
        })?;
        // Feed stdin on a thread to avoid deadlock with large outputs.
        let writer = stdin.map(|s| {
            let mut pipe = child.stdin.take().expect("piped stdin");
            let data = s.as_bytes().to_vec();
            std::thread::spawn(move || {
                let _ = pipe.write_all(&data);
            })
        });
        let (status, out_bytes, err_bytes) = match stall_limit {
            None => {
                let out = child.wait_with_output().map_err(|e| GitError::Failed {
                    message: e.to_string(),
                    stderr: String::new(),
                })?;
                (out.status, out.stdout, out.stderr)
            }
            Some(limit) => wait_with_stall_limit(child, limit)?,
        };
        if let Some(w) = writer {
            let _ = w.join();
        }
        let stdout = String::from_utf8_lossy(&out_bytes).into_owned();
        let code = status.code();
        if code.is_some_and(|c| ok.contains(&c)) {
            return Ok(stdout);
        }
        let stderr = strip_progress(&String::from_utf8_lossy(&err_bytes));
        if stderr.contains("not a git repository") {
            return Err(GitError::NotARepo {
                message: stderr
                    .lines()
                    .next()
                    .unwrap_or("Not a git repository")
                    .to_string(),
            });
        }
        let message = stderr
            .lines()
            .next()
            .map(str::to_string)
            .unwrap_or_else(|| {
                format!(
                    "git exited with status {}",
                    code.map_or("signal".into(), |c| c.to_string())
                )
            });
        Err(GitError::Failed { message, stderr })
    }
}

/// Removes `--progress` lines (counters redrawn with `\r`) so errors show only git's message.
fn strip_progress(stderr: &str) -> String {
    const PROGRESS: &[&str] = &[
        "Enumerating objects:",
        "Counting objects:",
        "Compressing objects:",
        "Writing objects:",
        "Receiving objects:",
        "Resolving deltas:",
        "Unpacking objects:",
        "Finding sources:",
        "Updating files:",
        "Checking connectivity",
        "Delta compression using",
    ];
    let is_progress = |line: &str| {
        let l = line.strip_prefix("remote: ").unwrap_or(line).trim_start();
        PROGRESS.iter().any(|p| l.starts_with(p))
            || l.strip_prefix("Total ")
                .is_some_and(|r| r.starts_with(|c: char| c.is_ascii_digit()))
    };
    let mut out = String::with_capacity(stderr.len());
    for line in stderr.split_inclusive('\n') {
        let (body, nl) = match line.strip_suffix('\n') {
            Some(b) => (b, "\n"),
            None => (line, ""),
        };
        // A redrawn counter: what a terminal would finally show is the text after the last `\r`.
        let shown = body
            .trim_end_matches('\r')
            .rsplit('\r')
            .next()
            .unwrap_or("");
        if !is_progress(shown) {
            out.push_str(shown);
            out.push_str(nl);
        }
    }
    out
}

/// Milliseconds since `start`, as stored in the shared last-output clock.
fn millis_since(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// Reads a pipe to the end on a thread; the receiver yields the bytes. Every chunk read
/// stamps `last_output` (milliseconds since `start`).
fn drain<R: Read + Send + 'static>(
    pipe: Option<R>,
    last_output: Arc<AtomicU64>,
    start: Instant,
) -> mpsc::Receiver<Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    if let Some(mut p) = pipe {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let mut chunk = [0u8; 8192];
            loop {
                match p.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => {
                        buf.extend_from_slice(&chunk[..n]);
                        last_output.store(millis_since(start), Ordering::Relaxed);
                    }
                    Err(e) if e.kind() == ErrorKind::Interrupted => {}
                    Err(_) => break,
                }
            }
            let _ = tx.send(buf);
        });
    }
    rx
}

fn signal_group(child: &Child, signal: &str) {
    let _ = Command::new("kill")
        .args([signal, "--", &format!("-{}", child.id())])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

/// Stops the child's whole process group (it was spawned as the group leader): SIGTERM first,
/// so git can remove its `index.lock` / ref locks, then SIGKILL if it is still running.
fn stop_group(child: &mut Child) {
    signal_group(child, "-TERM");
    let deadline = Instant::now() + TERM_GRACE;
    while Instant::now() < deadline {
        if matches!(child.try_wait(), Ok(Some(_))) {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    signal_group(child, "-KILL");
    let _ = child.kill();
    let _ = child.wait();
}

/// Kills whatever is left of the group once git itself has exited (a helper holding a pipe).
fn kill_group(child: &mut Child) {
    signal_group(child, "-KILL");
    let _ = child.kill();
    let _ = child.wait();
}

fn human(d: Duration) -> String {
    let s = d.as_secs();
    if s >= 120 && s.is_multiple_of(60) {
        format!("{} min", s / 60)
    } else {
        format!("{s}s")
    }
}

/// Same try_wait pattern as `env::login_env_from`. Pipes are read on threads so a full pipe
/// cannot stall the child, and a helper that outlives git while holding a pipe cannot stall
/// us: after a short grace it is killed and git's exit status stands.
fn wait_with_stall_limit(
    mut child: Child,
    limit: Duration,
) -> GitResult<(ExitStatus, Vec<u8>, Vec<u8>)> {
    let start = Instant::now();
    let last_output = Arc::new(AtomicU64::new(0));
    let out = drain(child.stdout.take(), last_output.clone(), start);
    let err = drain(child.stderr.take(), last_output.clone(), start);
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) => {
                let quiet = millis_since(start).saturating_sub(last_output.load(Ordering::Relaxed));
                if Duration::from_millis(quiet) >= limit {
                    stop_group(&mut child);
                    return Err(GitError::Failed {
                        message: format!("No progress for {}, stopped", human(limit)),
                        stderr: String::new(),
                    });
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => {
                stop_group(&mut child);
                return Err(GitError::Failed {
                    message: e.to_string(),
                    stderr: String::new(),
                });
            }
        }
    };
    // git has exited, so its own output is complete; only a straggler can keep a pipe open.
    let grace = Instant::now() + Duration::from_secs(2);
    let rest = |rx: &mpsc::Receiver<Vec<u8>>| {
        rx.recv_timeout(grace.saturating_duration_since(Instant::now()))
    };
    let (o, e) = (rest(&out), rest(&err));
    // A pipe still open means a group member is alive, so the pgid cannot have been reused.
    let timed_out = |r: &Result<Vec<u8>, mpsc::RecvTimeoutError>| {
        matches!(r, Err(mpsc::RecvTimeoutError::Timeout))
    };
    if timed_out(&o) || timed_out(&e) {
        kill_group(&mut child);
    }
    Ok((status, o.unwrap_or_default(), e.unwrap_or_default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_lines_are_removed_from_errors() {
        let stderr = "Enumerating objects: 1, done.\nCounting objects:  50% (1/2)\rCounting objects: 100% (2/2), done.\n\
                      Writing objects: 100% (3/3), 250 bytes | 250.00 KiB/s, done.\nTotal 3 (delta 0), reused 0 (delta 0)\n\
                      remote: Resolving deltas: 100% (1/1)\n\
                      To github.com:o/r.git\n ! [rejected]        main -> main (fetch first)\n\
                      error: failed to push some refs\n";
        assert_eq!(
            strip_progress(stderr),
            "To github.com:o/r.git\n ! [rejected]        main -> main (fetch first)\nerror: failed to push some refs\n"
        );
    }

    #[test]
    fn ordinary_errors_are_unchanged() {
        let s = "fatal: couldn't find remote ref nope\nTotal recall\n";
        assert_eq!(strip_progress(s), s);
    }

    #[test]
    fn stall_limit_reads_in_minutes_when_long() {
        assert_eq!(human(Duration::from_secs(300)), "5 min");
        assert_eq!(human(Duration::from_secs(1)), "1s");
    }
}
