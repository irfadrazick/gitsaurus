mod common;
use common::TestRepo;
use gitsaurus_lib::git::ops::remote::*;
use gitsaurus_lib::git::{Git, GitError, OpOutcome, PullMode};

/// Bare remote plus two clones (a, b) sharing one initial commit on main.
fn setup() -> (TestRepo, TestRepo, TestRepo) {
    let remote = TestRepo::bare();
    let seed = TestRepo::clone_of(&remote);
    seed.write("f.txt", "base\n");
    seed.commit_all("base");
    seed.git(&["push", "origin", "HEAD:main"]);
    let a = TestRepo::clone_of(&remote);
    let b = TestRepo::clone_of(&remote);
    (remote, a, b)
}

fn parents(r: &TestRepo) -> usize {
    r.git(&["rev-list", "--parents", "-n", "1", "HEAD"])
        .split_whitespace()
        .count()
        - 1
}

/// a pushes "a.txt"; b has a local commit "b.txt" (diverged).
fn diverge(a: &TestRepo, b: &TestRepo) {
    a.write("a.txt", "a\n");
    a.commit_all("a change");
    a.git(&["push", "origin", "main"]);
    b.write("b.txt", "b\n");
    b.commit_all("b change");
}

#[test]
fn fetch_updates_remote_branch() {
    let (_r, a, b) = setup();
    a.write("a.txt", "a\n");
    let oid = a.commit_all("a change");
    a.git(&["push", "origin", "main"]);
    fetch_all(&Git::new(b.path())).unwrap();
    assert_eq!(b.git(&["rev-parse", "origin/main"]).trim(), oid);
}

#[test]
fn pull_default_merges_when_diverged_and_no_config() {
    let (_r, a, b) = setup();
    diverge(&a, &b);
    let out = pull(&Git::new(b.path()), PullMode::Default).unwrap();
    assert_eq!(out, OpOutcome::Done);
    assert_eq!(parents(&b), 2);
}

#[test]
fn pull_default_respects_pull_rebase_true() {
    let (_r, a, b) = setup();
    diverge(&a, &b);
    b.git(&["config", "pull.rebase", "true"]);
    let out = pull(&Git::new(b.path()), PullMode::Default).unwrap();
    assert_eq!(out, OpOutcome::Done);
    assert_eq!(parents(&b), 1);
}

#[test]
fn pull_ff_only_fails_when_diverged() {
    let (_r, a, b) = setup();
    diverge(&a, &b);
    let err = pull(&Git::new(b.path()), PullMode::FfOnly).unwrap_err();
    assert!(matches!(err, GitError::Failed { .. }), "{err:?}");
    assert_eq!(parents(&b), 1);
}

#[test]
fn pull_conflict_returns_conflicts() {
    let (_r, a, b) = setup();
    a.write("f.txt", "from a\n");
    a.commit_all("a edit");
    a.git(&["push", "origin", "main"]);
    b.write("f.txt", "from b\n");
    b.commit_all("b edit");
    let out = pull(&Git::new(b.path()), PullMode::Default).unwrap();
    assert_eq!(out, OpOutcome::Conflicts);
}

#[test]
fn pull_branch_fast_forwards_non_checked_out_branch() {
    let (_r, a, b) = setup();
    b.git(&["checkout", "-b", "other"]);
    a.write("a.txt", "a\n");
    let oid = a.commit_all("a change");
    a.git(&["push", "origin", "main"]);
    let g = Git::new(b.path());
    pull_branch(&g, "main").unwrap();
    assert_eq!(b.git(&["rev-parse", "main"]).trim(), oid);
    assert_eq!(
        b.git(&["rev-parse", "--abbrev-ref", "HEAD"]).trim(),
        "other"
    );
}

#[test]
fn pull_branch_checked_out_uses_ff_only_and_errors_without_upstream() {
    let (_r, a, b) = setup();
    a.write("a.txt", "a\n");
    let oid = a.commit_all("a change");
    a.git(&["push", "origin", "main"]);
    let g = Git::new(b.path());
    pull_branch(&g, "main").unwrap();
    assert_eq!(b.git(&["rev-parse", "HEAD"]).trim(), oid);
    b.git(&["branch", "lonely"]);
    let err = pull_branch(&g, "lonely").unwrap_err();
    assert!(err.to_string().contains("No upstream"), "{err}");
}

#[test]
fn pull_branch_non_fast_forward_fails_without_forcing() {
    let (_r, a, b) = setup();
    b.git(&["branch", "--track", "side", "origin/main"]);
    b.git(&["checkout", "side"]);
    b.write("b.txt", "b\n");
    let local = b.commit_all("b side");
    b.git(&["checkout", "main"]);
    a.write("a.txt", "a\n");
    a.commit_all("a change");
    a.git(&["push", "origin", "main"]);
    b.git(&["fetch"]);
    let err = pull_branch(&Git::new(b.path()), "side").unwrap_err();
    assert!(matches!(err, GitError::Failed { .. }), "{err:?}");
    assert_eq!(b.git(&["rev-parse", "side"]).trim(), local);
}

#[test]
fn push_new_branch_sets_upstream() {
    let (remote, _a, b) = setup();
    b.git(&["checkout", "-b", "feature/x"]);
    b.write("x.txt", "x\n");
    let oid = b.commit_all("x");
    push(&Git::new(b.path()), "feature/x", false).unwrap();
    assert_eq!(remote.git(&["rev-parse", "feature/x"]).trim(), oid);
    assert_eq!(
        b.git(&["rev-parse", "--abbrev-ref", "feature/x@{upstream}"])
            .trim(),
        "origin/feature/x"
    );
}

#[test]
fn push_without_remote_fails() {
    let r = TestRepo::new();
    r.write("f.txt", "x\n");
    r.commit_all("x");
    let err = push(&Git::new(r.path()), "main", false).unwrap_err();
    assert!(err.to_string().contains("No remote configured"), "{err}");
}

#[test]
fn push_rejected_maps_to_push_rejected_and_force_does_not_clobber_fetched_commits() {
    let (remote, a, b) = setup();
    a.write("a.txt", "a\n");
    let theirs = a.commit_all("a change");
    a.git(&["push", "origin", "main"]);
    b.write("b.txt", "b\n");
    b.commit_all("b change");
    let g = Git::new(b.path());
    let err = push(&g, "main", false).unwrap_err();
    assert!(matches!(err, GitError::PushRejected { .. }), "{err:?}");
    // A (background) fetch moves origin/main, which alone would satisfy
    // --force-with-lease; --force-if-includes still refuses, because b never
    // integrated the fetched commit.
    fetch_all(&g).unwrap();
    let err = push(&g, "main", true).unwrap_err();
    assert!(matches!(err, GitError::PushRejected { .. }), "{err:?}");
    assert_eq!(remote.git(&["rev-parse", "main"]).trim(), theirs);
}

#[test]
fn force_push_rewrites_history_the_local_branch_has_seen() {
    let (remote, _a, b) = setup();
    b.write("b.txt", "b\n");
    b.commit_all("b change");
    b.git(&["push", "origin", "main"]);
    b.write("b.txt", "b amended\n");
    b.git(&["commit", "-a", "--amend", "-m", "b change (amended)"]);
    let oid = b.git(&["rev-parse", "HEAD"]).trim().to_string();
    let g = Git::new(b.path());
    let err = push(&g, "main", false).unwrap_err();
    assert!(matches!(err, GitError::PushRejected { .. }), "{err:?}");
    push(&g, "main", true).unwrap();
    assert_eq!(remote.git(&["rev-parse", "main"]).trim(), oid);
}

#[test]
fn push_tag_and_set_upstream() {
    let (remote, _a, b) = setup();
    b.git(&["tag", "v1"]);
    let g = Git::new(b.path());
    push_tag(&g, "v1", None).unwrap();
    assert!(!remote.git(&["tag", "-l", "v1"]).trim().is_empty());

    b.git(&["branch", "loose"]);
    set_upstream(&g, "loose", "origin/main").unwrap();
    assert_eq!(
        b.git(&["rev-parse", "--abbrev-ref", "loose@{upstream}"])
            .trim(),
        "origin/main"
    );
    assert!(set_upstream(&g, "-bad", "origin/main").is_err());
}
