#![allow(dead_code)]
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Hermetic temporary git repository for integration tests.
pub struct TestRepo {
    dir: TempDir,
}

fn base_cmd(dir: &Path) -> Command {
    let mut c = Command::new("git");
    c.current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C");
    c
}

fn run_in(dir: &Path, args: &[&str]) -> String {
    let out = base_cmd(dir).args(args).output().expect("spawn git");
    assert!(
        out.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

impl TestRepo {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        run_in(dir.path(), &["init", "-b", "main"]);
        run_in(dir.path(), &["config", "user.name", "Test User"]);
        run_in(dir.path(), &["config", "user.email", "test@example.com"]);
        run_in(dir.path(), &["config", "commit.gpgsign", "false"]);
        TestRepo { dir }
    }

    pub fn bare() -> Self {
        let dir = tempfile::tempdir().unwrap();
        run_in(dir.path(), &["init", "--bare", "-b", "main"]);
        TestRepo { dir }
    }

    pub fn clone_of(other: &TestRepo) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let src = other.path().to_str().unwrap();
        let dst = dir.path().to_str().unwrap();
        run_in(dir.path(), &["clone", src, dst]);
        run_in(dir.path(), &["config", "user.name", "Test User"]);
        run_in(dir.path(), &["config", "user.email", "test@example.com"]);
        run_in(dir.path(), &["config", "commit.gpgsign", "false"]);
        TestRepo { dir }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn write(&self, rel: &str, contents: &str) {
        let p: PathBuf = self.path().join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(p, contents).unwrap();
    }

    /// Runs git, panics on failure, returns stdout.
    pub fn git(&self, args: &[&str]) -> String {
        run_in(self.path(), args)
    }

    /// Runs git, ignoring a non-zero exit (e.g. a conflicting merge); returns stdout.
    pub fn git_allow_fail(&self, args: &[&str]) -> String {
        let out = base_cmd(self.path())
            .args(args)
            .output()
            .expect("spawn git");
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    /// Stages everything, commits, returns the new commit oid.
    pub fn commit_all(&self, msg: &str) -> String {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-m", msg]);
        self.git(&["rev-parse", "HEAD"]).trim().to_string()
    }
}
