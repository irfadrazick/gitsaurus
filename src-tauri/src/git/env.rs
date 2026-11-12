use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::sync::mpsc;
use std::time::{Duration, Instant};

const KEYS: [&str; 2] = ["PATH", "SSH_AUTH_SOCK"];
const LOGIN_SHELL_TIMEOUT: Duration = Duration::from_secs(3);

static ENV: OnceLock<Vec<(String, String)>> = OnceLock::new();

/// PATH and SSH_AUTH_SOCK from the user's login shell (cached).
pub fn git_env() -> &'static [(String, String)] {
    ENV.get_or_init(|| login_shell_env().unwrap_or_else(fallback_env))
}

fn login_shell_env() -> Option<Vec<(String, String)>> {
    let shell = std::env::var("SHELL").ok().filter(|s| !s.is_empty())?;
    login_env_from(Path::new(&shell), LOGIN_SHELL_TIMEOUT)
}

/// Runs `shell -ilc 'env -0'` and extracts PATH / SSH_AUTH_SOCK. Returns None on
/// failure or if the whole operation (including reading stdout) exceeds `timeout`.
pub(crate) fn login_env_from(shell: &Path, timeout: Duration) -> Option<Vec<(String, String)>> {
    let mut child = Command::new(shell)
        .args(["-ilc", "env -0"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .ok()?;
    let pgid = child.id();
    let mut stdout = child.stdout.take()?;
    // Read on a thread; a grandchild holding the pipe open must not block us.
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        let _ = tx.send(buf);
    });
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break Some(s),
            Ok(None) if Instant::now() >= deadline => break None,
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(_) => break None,
        }
    };
    let ok = status.is_some_and(|s| s.success());
    if status.is_none() {
        let _ = child.kill();
        let _ = child.wait();
    }
    // Kill any stragglers (grandchildren) in the shell's process group.
    let _ = Command::new("kill")
        .args(["-9", &format!("-{pgid}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if !ok {
        return None;
    }
    let buf = rx
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        .ok()?;
    let text = String::from_utf8_lossy(&buf);
    let vars: Vec<(String, String)> = text
        .split('\0')
        .filter_map(|kv| kv.split_once('='))
        .filter(|(k, _)| KEYS.contains(k))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    vars.iter()
        .any(|(k, v)| k == "PATH" && !v.is_empty())
        .then_some(vars)
}

fn fallback_env() -> Vec<(String, String)> {
    let extra = "/opt/homebrew/bin:/usr/local/bin";
    let path = match std::env::var("PATH") {
        Ok(p) if !p.is_empty() => format!("{extra}:{p}"),
        _ => format!("{extra}:/usr/bin:/bin:/usr/sbin:/sbin"),
    };
    let mut v = vec![("PATH".to_string(), path)];
    if let Ok(s) = std::env::var("SSH_AUTH_SOCK") {
        v.push(("SSH_AUTH_SOCK".to_string(), s));
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn script(dir: &Path, body: &str) -> std::path::PathBuf {
        let p = dir.join("fakeshell.sh");
        std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        p
    }

    fn assert_none_fast(body: &str) {
        let d = tempfile::tempdir().unwrap();
        let sh = script(d.path(), body);
        let t = Duration::from_millis(500);
        let start = Instant::now();
        assert_eq!(login_env_from(&sh, t), None);
        assert!(
            start.elapsed() < t + Duration::from_secs(1),
            "took {:?}",
            start.elapsed()
        );
    }

    #[test]
    fn grandchild_holding_pipe_does_not_hang() {
        assert_none_fast("sleep 30 &\nexit 0");
    }

    #[test]
    fn hung_shell_is_killed_at_timeout() {
        assert_none_fast("sleep 30");
    }

    #[test]
    fn parses_path_from_shell_output() {
        let d = tempfile::tempdir().unwrap();
        let sh = script(d.path(), "printf 'PATH=/x\\0'");
        assert_eq!(
            login_env_from(&sh, Duration::from_secs(3)),
            Some(vec![("PATH".to_string(), "/x".to_string())])
        );
    }
}
