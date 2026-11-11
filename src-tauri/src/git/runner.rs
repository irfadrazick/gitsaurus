use super::env::git_env;
use super::error::{GitError, GitResult};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
        self.exec(args, &[0], None, &[])
    }

    /// Read-only command: avoids taking optional index locks.
    pub fn run_read(&self, args: &[&str]) -> GitResult<String> {
        self.exec(args, &[0], None, &[("GIT_OPTIONAL_LOCKS", "0")])
    }

    pub fn run_ok_codes(&self, args: &[&str], ok: &[i32]) -> GitResult<String> {
        self.exec(args, ok, None, &[])
    }

    pub fn run_stdin(&self, args: &[&str], stdin: &str) -> GitResult<String> {
        self.exec(args, &[0], Some(stdin), &[])
    }

    pub fn run_env(&self, args: &[&str], extra_env: &[(&str, &str)]) -> GitResult<String> {
        self.exec(args, &[0], None, extra_env)
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
        let out = child.wait_with_output().map_err(|e| GitError::Failed {
            message: e.to_string(),
            stderr: String::new(),
        })?;
        if let Some(w) = writer {
            let _ = w.join();
        }
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let code = out.status.code();
        if code.is_some_and(|c| ok.contains(&c)) {
            return Ok(stdout);
        }
        let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
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
