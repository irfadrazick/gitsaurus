use std::path::Path;
use std::process::Command;

use crate::git::{GitError, GitResult};

fn spawn_err(e: std::io::Error) -> GitError {
    GitError::Failed {
        message: "could not launch external program".into(),
        stderr: e.to_string(),
    }
}

pub fn open_terminal(repo: &Path) -> GitResult<()> {
    Command::new("open")
        .args(["-a", "Terminal"])
        .arg(repo)
        .spawn()
        .map(|_| ())
        .map_err(spawn_err)
}

pub fn reveal_in_finder(path: &Path) -> GitResult<()> {
    Command::new("open")
        .arg("-R")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(spawn_err)
}

/// `path` is repo-relative. Empty `editor_command` uses the default app.
pub fn open_in_editor(repo: &Path, path: &str, editor_command: &str) -> GitResult<()> {
    let abs = repo.join(path);
    let cmd = editor_command.trim();
    let mut c = if cmd.is_empty() {
        let mut c = Command::new("open");
        c.arg(&abs);
        c
    } else {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
        let mut c = Command::new(shell);
        // The path is passed as $1, never interpolated into the script.
        c.arg("-lc").arg(format!("{cmd} \"$1\"")).arg("_").arg(&abs);
        c
    };
    c.current_dir(repo).spawn().map(|_| ()).map_err(spawn_err)
}
