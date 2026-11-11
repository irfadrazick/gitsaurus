use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
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
    let mut child = Command::new(shell)
        .args(["-ilc", "env -0"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    // Drain on a thread so a full pipe never stalls the child.
    let reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        buf
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break Some(s),
            Ok(None) if start.elapsed() >= LOGIN_SHELL_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(_) => break None,
        }
    };
    let buf = reader.join().ok()?;
    if !status?.success() {
        return None;
    }
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
