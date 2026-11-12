mod common;
use common::TestRepo;
use gitsaurus_lib::git::{Git, GitError};
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant};

/// A repo whose `origin` is an ssh remote served by a "transport" that never answers.
fn hanging_remote() -> (TestRepo, tempfile::TempDir) {
    let r = TestRepo::new();
    let bin = tempfile::tempdir().unwrap();
    let ssh = bin.path().join("hang-ssh.sh");
    std::fs::write(&ssh, "#!/bin/sh\nsleep 30\n").unwrap();
    std::fs::set_permissions(&ssh, std::fs::Permissions::from_mode(0o755)).unwrap();
    r.git(&["remote", "add", "origin", "ssh://example.invalid/repo.git"]);
    r.git(&["config", "core.sshCommand", ssh.to_str().unwrap()]);
    (r, bin)
}

#[test]
fn silent_network_command_is_stopped_after_the_stall_limit() {
    let (r, _bin) = hanging_remote();
    let start = Instant::now();
    let err = Git::new(r.path())
        .run_with_stall_limit(&["fetch", "origin"], Duration::from_secs(1))
        .unwrap_err();
    assert!(
        start.elapsed() < Duration::from_secs(6),
        "took {:?}",
        start.elapsed()
    );
    match err {
        GitError::Failed { message, .. } => assert_eq!(message, "No progress for 1s, stopped"),
        other => panic!("{other:?}"),
    }
}

/// Runs `script` through a shell alias so a test can control timing and output.
fn alias(script: &str) -> [String; 3] {
    ["-c".into(), format!("alias.t=!{script}"), "t".into()]
}

#[test]
fn command_that_keeps_reporting_is_not_stopped() {
    let r = TestRepo::new();
    // 2.4s in total, far past the 1s limit, but never silent for more than 0.3s.
    let a = alias("for i in 1 2 3 4 5 6 7 8; do echo working >&2; sleep 0.3; done; echo done");
    let args: Vec<&str> = a.iter().map(String::as_str).collect();
    let out = Git::new(r.path())
        .run_with_stall_limit(&args, Duration::from_secs(1))
        .unwrap();
    assert_eq!(out.trim(), "done");
}

#[test]
fn a_stalled_command_gets_sigterm_before_sigkill() {
    // git removes its lock files on SIGTERM; SIGKILL leaves them behind.
    let r = TestRepo::new();
    let a = alias("trap 'touch got-term; exit 143' TERM; sleep 30 & wait");
    let args: Vec<&str> = a.iter().map(String::as_str).collect();
    let err = Git::new(r.path())
        .run_with_stall_limit(&args, Duration::from_secs(1))
        .unwrap_err();
    assert!(matches!(err, GitError::Failed { .. }), "{err:?}");
    assert!(
        r.path().join("got-term").exists(),
        "the command was not sent SIGTERM"
    );
}

#[test]
fn timeout_run_returns_output_and_errors_like_run() {
    let r = TestRepo::new();
    r.write("f.txt", "x\n");
    let oid = r.commit_all("c");
    let g = Git::new(r.path());
    let out = g
        .run_with_stall_limit(&["rev-parse", "HEAD"], Duration::from_secs(30))
        .unwrap();
    assert_eq!(out.trim(), oid);
    let err = g
        .run_with_stall_limit(&["rev-parse", "--verify", "nope"], Duration::from_secs(30))
        .unwrap_err();
    assert!(matches!(err, GitError::Failed { .. }), "{err:?}");
}
