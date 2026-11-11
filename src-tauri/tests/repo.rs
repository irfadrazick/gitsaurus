mod common;
use common::TestRepo;
use gitsaurus_lib::git::{Git, GitError, git_env, open_repo};
use std::path::PathBuf;

#[test]
fn open_repo_from_subdir_returns_toplevel() {
    let r = TestRepo::new();
    r.write("a/b.txt", "x");
    let info = open_repo(&r.path().join("a")).unwrap();
    assert_eq!(
        PathBuf::from(&info.path).canonicalize().unwrap(),
        r.path().canonicalize().unwrap()
    );
    assert_eq!(info.name, r.path().file_name().unwrap().to_str().unwrap());
}

#[test]
fn open_repo_on_plain_dir_is_not_a_repo() {
    let d = tempfile::tempdir().unwrap();
    assert!(matches!(
        open_repo(d.path()),
        Err(GitError::NotARepo { .. })
    ));
}

#[test]
fn deleted_repo_dir_is_not_a_repo() {
    let r = TestRepo::new();
    let p = r.path().to_path_buf();
    std::fs::remove_dir_all(&p).unwrap();
    assert!(matches!(
        Git::new(&p).run(&["status"]),
        Err(GitError::NotARepo { .. })
    ));
}

#[test]
fn failed_command_carries_stderr() {
    let r = TestRepo::new();
    match Git::new(r.path()).run(&["checkout", "nope"]) {
        Err(GitError::Failed { stderr, .. }) => assert!(stderr.contains("nope")),
        e => panic!("{e:?}"),
    }
}

#[test]
fn git_env_has_path() {
    assert!(
        git_env()
            .iter()
            .any(|(k, v)| k == "PATH" && v.contains("/usr/bin"))
    );
}
