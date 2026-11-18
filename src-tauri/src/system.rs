use std::path::Path;
use std::process::Command;

use crate::git::{GitError, GitResult};

fn spawn_err(e: std::io::Error) -> GitError {
    GitError::Failed {
        message: "could not launch external program".into(),
        stderr: e.to_string(),
    }
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

/// The `.app` bundle that holds `exe` (`<bundle>/Contents/MacOS/<binary>`), or `None` when the app
/// isn't running from a bundle (`tauri dev`).
pub fn bundle_of(exe: &Path) -> Option<&Path> {
    let macos = exe.parent()?;
    let contents = macos.parent()?;
    let bundle = contents.parent()?;
    (macos.file_name()? == "MacOS"
        && contents.file_name()? == "Contents"
        && bundle.extension()? == "app")
        .then_some(bundle)
}

/// How long the relauncher waits for the old copy to exit before opening the new one anyway.
const RELAUNCH_WAIT_TICKS: u32 = 100; // of 0.1 s

/// A detached shell that waits for process `pid` to exit, then opens `bundle` through LaunchServices.
///
/// After an update, Restart must not start the new copy while the old one still runs (Tauri's own
/// restart uses `open -n`): the Dock then gives the new copy a tile of its own, next to the one the
/// user keeps there, for as long as it runs. Opened only after the old copy has gone, it takes the
/// kept tile like any normal launch.
fn relaunch_cmd(pid: u32, bundle: &Path) -> Command {
    let mut c = Command::new("/bin/sh");
    // The pid and path are passed as $1 and $2, never interpolated into the script.
    c.arg("-c")
        .arg(format!(
            "i=0; while kill -0 \"$1\" 2>/dev/null && [ $i -lt {RELAUNCH_WAIT_TICKS} ]; do sleep 0.1; i=$((i+1)); done; exec /usr/bin/open \"$2\""
        ))
        .arg("_")
        .arg(pid.to_string())
        .arg(bundle)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    // Its own process group, so it outlives this app.
    std::os::unix::process::CommandExt::process_group(&mut c, 0);
    c
}

/// Starts the relauncher for `bundle`, which opens it once this process has exited.
pub fn relaunch_after_exit(bundle: &Path) -> GitResult<()> {
    relaunch_cmd(std::process::id(), bundle)
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
    fn bundle_of_finds_the_app_bundle() {
        let exe = Path::new("/Applications/Gitsaurus.app/Contents/MacOS/gitsaurus");
        assert_eq!(
            bundle_of(exe),
            Some(Path::new("/Applications/Gitsaurus.app"))
        );
        assert_eq!(
            bundle_of(Path::new("/repo/src-tauri/target/debug/gitsaurus")),
            None
        );
        assert_eq!(bundle_of(Path::new("/x/Contents/MacOS/gitsaurus")), None);
        assert_eq!(bundle_of(Path::new("gitsaurus")), None);
    }

    #[test]
    fn relauncher_waits_for_the_pid_then_opens_the_bundle_without_n() {
        let c = relaunch_cmd(42, Path::new("/Applications/Git saurus.app"));
        assert_eq!(c.get_program(), "/bin/sh");
        let args: Vec<&OsStr> = c.get_args().collect();
        assert_eq!(args[0], "-c");
        let script = args[1].to_str().unwrap();
        assert!(script.contains("kill -0 \"$1\""));
        assert!(script.ends_with("exec /usr/bin/open \"$2\""));
        assert!(!script.contains("-n"));
        assert_eq!(&args[2..], ["_", "42", "/Applications/Git saurus.app"]);
    }

    #[test]
    fn relauncher_opens_the_bundle_once_the_process_has_exited() {
        // Run the real script with `open` swapped for a marker file, against a short-lived process.
        let dir = std::env::temp_dir().join(format!("gitsaurus-relaunch-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let marker = dir.join("opened");
        let mut sleeper = Command::new("/bin/sleep").arg("0.3").spawn().unwrap();
        let c = relaunch_cmd(sleeper.id(), &marker);
        let args: Vec<String> = c
            .get_args()
            .map(|a| a.to_str().unwrap().to_owned())
            .collect();
        let script = args[1].replace("exec /usr/bin/open", "touch");
        let mut run = Command::new("/bin/sh")
            .arg("-c")
            .arg(script)
            .args(&args[2..])
            .spawn()
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(!marker.exists(), "opened before the process exited");
        sleeper.wait().unwrap();
        run.wait().unwrap();
        assert!(marker.exists());
        std::fs::remove_dir_all(&dir).ok();
    }

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
