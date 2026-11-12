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

/// `path` is repo-relative. Empty `editor_command` uses the default text editor.
pub fn open_in_editor(repo: &Path, path: &str, editor_command: &str) -> GitResult<()> {
    editor_cmd(repo, path, editor_command)
        .spawn()
        .map(|_| ())
        .map_err(spawn_err)
}

fn editor_cmd(repo: &Path, path: &str, editor_command: &str) -> Command {
    let abs = repo.join(path);
    let cmd = editor_command.trim();
    let mut c = if cmd.is_empty() {
        // `open -t`: always the default *text editor*, so a repo file such as an `.app`
        // bundle or a `.command` script is never launched.
        let mut c = Command::new("open");
        c.arg("-t").arg(&abs);
        c
    } else {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
        let mut c = Command::new(shell);
        // The path is passed as $1, never interpolated into the script.
        c.arg("-lc").arg(format!("{cmd} \"$1\"")).arg("_").arg(&abs);
        c
    };
    c.current_dir(repo);
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn empty_editor_command_opens_in_the_default_text_editor() {
        let c = editor_cmd(Path::new("/r"), "run.command", "  ");
        assert_eq!(c.get_program(), "open");
        let args: Vec<&OsStr> = c.get_args().collect();
        assert_eq!(args, ["-t", "/r/run.command"]);
    }

    #[test]
    fn editor_command_gets_the_path_as_an_argument() {
        let c = editor_cmd(Path::new("/r"), "a b.txt", "code -w");
        let args: Vec<&OsStr> = c.get_args().collect();
        assert_eq!(args, ["-lc", "code -w \"$1\"", "_", "/r/a b.txt"]);
    }
}
