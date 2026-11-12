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
fn network_command_is_killed_at_timeout() {
    let (r, _bin) = hanging_remote();
    let start = Instant::now();
    let err = Git::new(r.path())
        .run_timeout(&["fetch", "origin"], Duration::from_secs(1))
        .unwrap_err();
    assert!(
        start.elapsed() < Duration::from_secs(5),
        "took {:?}",
        start.elapsed()
    );
    match err {
        GitError::Failed { message, .. } => assert_eq!(message, "Timed out after 1s"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn timeout_run_returns_output_and_errors_like_run() {
    let r = TestRepo::new();
    r.write("f.txt", "x\n");
    let oid = r.commit_all("c");
    let g = Git::new(r.path());
    let out = g
        .run_timeout(&["rev-parse", "HEAD"], Duration::from_secs(30))
        .unwrap();
    assert_eq!(out.trim(), oid);
    let err = g
        .run_timeout(&["rev-parse", "--verify", "nope"], Duration::from_secs(30))
        .unwrap_err();
    assert!(matches!(err, GitError::Failed { .. }), "{err:?}");
}
