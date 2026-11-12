use super::env::git_env;
use super::error::{GitError, GitResult};
use std::io::{ErrorKind, Read, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// Hard limit for commands that talk to a remote (fetch, pull, push).
pub const NETWORK_TIMEOUT: Duration = Duration::from_secs(120);

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

    /// Like `run`, but the child (and anything it spawned, e.g. ssh) is killed
    /// after `timeout`, returning `Failed` "Timed out after Ns".
    pub fn run_timeout(&self, args: &[&str], timeout: Duration) -> GitResult<String> {
        self.exec(args, &[0], None, &[], Some(timeout))
    }

    /// A command that talks to a remote: `run` with `NETWORK_TIMEOUT`.
    pub fn run_network(&self, args: &[&str]) -> GitResult<String> {
        self.run_timeout(args, NETWORK_TIMEOUT)
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
        timeout: Option<Duration>,
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
        if timeout.is_some() {
            // Own process group, so a timeout can kill git's helpers (ssh, remote-https) too.
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
        let (status, out_bytes, err_bytes) = match timeout {
            None => {
                let out = child.wait_with_output().map_err(|e| GitError::Failed {
                    message: e.to_string(),
                    stderr: String::new(),
                })?;
                (out.status, out.stdout, out.stderr)
            }
            Some(t) => wait_with_timeout(child, t)?,
        };
        if let Some(w) = writer {
            let _ = w.join();
        }
        let stdout = String::from_utf8_lossy(&out_bytes).into_owned();
        let code = status.code();
        if code.is_some_and(|c| ok.contains(&c)) {
            return Ok(stdout);
        }
        let stderr = String::from_utf8_lossy(&err_bytes).into_owned();
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

/// Reads a pipe to the end on a thread; the receiver yields the bytes.
fn drain<R: Read + Send + 'static>(pipe: Option<R>) -> mpsc::Receiver<Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    if let Some(mut p) = pipe {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = p.read_to_end(&mut buf);
            let _ = tx.send(buf);
        });
    }
    rx
}

/// Kills the child's whole process group (it was spawned as the group leader).
fn kill_group(child: &mut Child) {
    let _ = Command::new("kill")
        .args(["-9", &format!("-{}", child.id())])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = child.kill();
    let _ = child.wait();
}

/// Same try_wait/kill pattern as `env::login_env_from`. Pipes are read on threads so a
/// full pipe cannot stall the child, and a helper that outlives git while holding a pipe
/// cannot stall us: after a short grace it is killed and git's exit status stands.
fn wait_with_timeout(
    mut child: Child,
    timeout: Duration,
) -> GitResult<(ExitStatus, Vec<u8>, Vec<u8>)> {
    let deadline = Instant::now() + timeout;
    let out = drain(child.stdout.take());
    let err = drain(child.stderr.take());
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) if Instant::now() >= deadline => {
                kill_group(&mut child);
                return Err(GitError::Failed {
                    message: format!("Timed out after {}s", timeout.as_secs()),
                    stderr: String::new(),
                });
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(e) => {
                kill_group(&mut child);
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
