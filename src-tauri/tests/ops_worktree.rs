mod common;
use common::TestRepo;
use gitsaurus_lib::git::ops::worktree::{
    commit, discard, last_commit_message, stage, stage_all, unstage, unstage_all,
};
use gitsaurus_lib::git::{Git, GitError, repo_state};

fn status_of(g: &Git, path: &str) -> Option<(String, String)> {
    repo_state(g)
        .unwrap()
        .files
        .into_iter()
        .find(|f| f.path == path)
        .map(|f| (f.index, f.worktree))
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

#[test]
fn stage_and_unstage_round_trip() {
    let r = TestRepo::new();
    r.write("a.txt", "1\n");
    r.commit_all("init");
    let g = Git::new(r.path());
    r.write("a.txt", "2\n");
    r.write("b.txt", "new\n");

    stage(&g, &s(&["a.txt"])).unwrap();
    assert_eq!(status_of(&g, "a.txt").unwrap().0, "M");
    assert_eq!(status_of(&g, "b.txt").unwrap().1, "?");
    unstage(&g, &s(&["a.txt"])).unwrap();
    assert_eq!(status_of(&g, "a.txt").unwrap().0, ".");

    // empty paths is a no-op, not "stage everything"
    stage(&g, &[]).unwrap();
    unstage(&g, &[]).unwrap();
    discard(&g, &[]).unwrap();
    assert_eq!(status_of(&g, "a.txt").unwrap().0, ".");

    stage_all(&g).unwrap();
    assert_eq!(status_of(&g, "a.txt").unwrap().0, "M");
    assert_eq!(status_of(&g, "b.txt").unwrap().0, "A");
    unstage_all(&g).unwrap();
    assert_eq!(status_of(&g, "a.txt").unwrap().0, ".");
    assert_eq!(status_of(&g, "b.txt").unwrap(), (".".into(), "?".into()));
}

#[test]
fn first_commit_in_fresh_repo_and_unstage_on_unborn_head() {
    let r = TestRepo::new();
    let g = Git::new(r.path());
    r.write("a.txt", "1\n");
    r.write("b.txt", "2\n");
    stage(&g, &s(&["a.txt", "b.txt"])).unwrap();
    assert_eq!(status_of(&g, "a.txt").unwrap().0, "A");
    unstage(&g, &s(&["a.txt"])).unwrap();
    assert_eq!(status_of(&g, "a.txt").unwrap(), (".".into(), "?".into()));
    assert_eq!(status_of(&g, "b.txt").unwrap().0, "A");
    unstage_all(&g).unwrap();
    assert_eq!(status_of(&g, "b.txt").unwrap(), (".".into(), "?".into()));

    assert!(matches!(
        last_commit_message(&g),
        Err(GitError::Failed { ref message, .. }) if message == "No commits yet"
    ));

    stage_all(&g).unwrap();
    commit(&g, "first", "", false).unwrap();
    assert_eq!(r.git(&["rev-list", "--count", "HEAD"]).trim(), "1");
    assert!(repo_state(&g).unwrap().files.is_empty());
}

#[test]
fn odd_paths_stage_diff_discard() {
    let r = TestRepo::new();
    r.write("a b.txt", "1\n");
    r.commit_all("init");
    let g = Git::new(r.path());
    r.write("a b.txt", "2\n");
    r.write("ünï.txt", "u\n");
    r.write("-dash.txt", "d\n");
    let all = s(&["a b.txt", "ünï.txt", "-dash.txt"]);

    stage(&g, &all).unwrap();
    for p in &all {
        assert_ne!(status_of(&g, p).unwrap().0, ".", "{p} should be staged");
    }
    unstage(&g, &all).unwrap();
    for p in &all {
        let (i, _) = status_of(&g, p).unwrap();
        assert_eq!(i, ".", "{p} should not be staged");
    }

    discard(&g, &all).unwrap();
    assert_eq!(
        std::fs::read_to_string(r.path().join("a b.txt")).unwrap(),
        "1\n"
    );
    assert!(!r.path().join("ünï.txt").exists());
    assert!(!r.path().join("-dash.txt").exists());
    assert!(repo_state(&g).unwrap().files.is_empty());
}

#[test]
fn commit_with_description_and_amend() {
    let r = TestRepo::new();
    r.write("a.txt", "1\n");
    let g = Git::new(r.path());
    stage_all(&g).unwrap();
    commit(&g, "  S  ", "D\n\n", false).unwrap();
    assert_eq!(r.git(&["log", "-1", "--format=%B"]), "S\n\nD\n\n");
    assert_eq!(
        last_commit_message(&g).unwrap(),
        ("S".to_string(), "D".to_string())
    );

    let before = r.git(&["rev-list", "--count", "HEAD"]);
    commit(&g, "S2", "", true).unwrap();
    assert_eq!(r.git(&["rev-list", "--count", "HEAD"]), before);
    assert_eq!(r.git(&["log", "-1", "--format=%s"]).trim(), "S2");
}

#[test]
fn blank_summary_rejected() {
    let r = TestRepo::new();
    r.write("a.txt", "1\n");
    let g = Git::new(r.path());
    stage_all(&g).unwrap();
    let err = commit(&g, "  \n", "desc", false).unwrap_err();
    assert!(
        matches!(err, GitError::Failed { ref message, .. } if message == "Commit summary is empty")
    );
}
