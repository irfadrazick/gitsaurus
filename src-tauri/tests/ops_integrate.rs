mod common;
use common::TestRepo;
use gitsaurus_lib::git::ops::integrate::*;
use gitsaurus_lib::git::{Git, OpOutcome, Side, repo_state};

/// main and `topic` both change f.txt after a common base.
fn diverged() -> TestRepo {
    let r = TestRepo::new();
    r.write("f.txt", "base\n");
    r.commit_all("base");
    r.git(&["checkout", "-b", "topic"]);
    r.write("f.txt", "topic\n");
    r.commit_all("topic change");
    r.git(&["checkout", "main"]);
    r.write("f.txt", "main\n");
    r.commit_all("main change");
    r
}

fn conflicted(g: &Git) -> Vec<String> {
    repo_state(g)
        .unwrap()
        .files
        .into_iter()
        .filter(|f| f.conflicted)
        .map(|f| f.path)
        .collect()
}

#[test]
fn merge_clean_returns_done() {
    let r = TestRepo::new();
    r.write("a.txt", "a\n");
    r.commit_all("base");
    r.git(&["checkout", "-b", "topic"]);
    r.write("b.txt", "b\n");
    r.commit_all("b");
    r.git(&["checkout", "main"]);
    r.write("c.txt", "c\n");
    r.commit_all("c");
    let g = Git::new(r.path());
    assert_eq!(merge(&g, "topic").unwrap(), OpOutcome::Done);
    assert_eq!(
        r.git(&["log", "-1", "--format=%P"])
            .split_whitespace()
            .count(),
        2
    );
}

#[test]
fn merge_rejects_option_like_target() {
    let r = diverged();
    let g = Git::new(r.path());
    assert!(merge(&g, "--abort").is_err());
}

#[test]
fn merge_conflict_returns_conflicts_then_resolve_theirs_and_continue() {
    let r = diverged();
    let g = Git::new(r.path());
    assert_eq!(merge(&g, "topic").unwrap(), OpOutcome::Conflicts);
    assert_eq!(conflicted(&g), vec!["f.txt"]);
    resolve_conflict(&g, "f.txt", Side::Theirs).unwrap();
    assert_eq!(continue_op(&g).unwrap(), OpOutcome::Done);
    assert_eq!(
        r.git(&["log", "-1", "--format=%P"])
            .split_whitespace()
            .count(),
        2
    );
    assert_eq!(
        std::fs::read_to_string(r.path().join("f.txt")).unwrap(),
        "topic\n"
    );
}

#[test]
fn rebase_two_step_conflicts_then_skip_and_abort() {
    let r = TestRepo::new();
    r.write("f.txt", "base\n");
    r.commit_all("base");
    r.git(&["checkout", "-b", "topic"]);
    r.write("f.txt", "t1\n");
    r.commit_all("t1");
    r.write("f.txt", "t2\n");
    r.commit_all("t2");
    r.git(&["checkout", "main"]);
    r.write("f.txt", "main\n");
    r.commit_all("main change");
    r.git(&["checkout", "topic"]);
    let orig = r.git(&["rev-parse", "HEAD"]);
    let g = Git::new(r.path());
    assert_eq!(rebase(&g, "main").unwrap(), OpOutcome::Conflicts);
    assert_eq!(skip_op(&g).unwrap(), OpOutcome::Conflicts);
    abort_op(&g).unwrap();
    assert_eq!(r.git(&["rev-parse", "HEAD"]), orig);
    assert!(repo_state(&g).unwrap().operation.is_none());
}

#[test]
fn rebase_resolve_and_continue_finishes() {
    let r = diverged();
    r.git(&["checkout", "topic"]);
    let g = Git::new(r.path());
    assert_eq!(rebase(&g, "main").unwrap(), OpOutcome::Conflicts);
    resolve_conflict(&g, "f.txt", Side::Ours).unwrap();
    assert_eq!(continue_op(&g).unwrap(), OpOutcome::Done);
    assert!(repo_state(&g).unwrap().operation.is_none());
}

#[test]
fn cherry_pick_and_revert_including_merge_commit() {
    let r = TestRepo::new();
    r.write("a.txt", "a\n");
    r.commit_all("base");
    r.git(&["checkout", "-b", "topic"]);
    r.write("b.txt", "b\n");
    let b = r.commit_all("b");
    r.git(&["checkout", "main"]);
    r.write("c.txt", "c\n");
    r.commit_all("c");
    let g = Git::new(r.path());
    assert_eq!(cherry_pick(&g, &b).unwrap(), OpOutcome::Done);
    assert!(r.path().join("b.txt").exists());
    assert_eq!(revert(&g, "HEAD").unwrap(), OpOutcome::Done);
    assert!(!r.path().join("b.txt").exists());

    // Merge commit: needs -m 1.
    r.git(&["reset", "--hard", "HEAD~2"]);
    r.git(&["merge", "--no-ff", "--no-edit", "topic"]);
    let m = r.git(&["rev-parse", "HEAD"]).trim().to_string();
    assert_eq!(revert(&g, &m).unwrap(), OpOutcome::Done);
    assert!(!r.path().join("b.txt").exists());
    r.git(&["reset", "--hard", &format!("{m}~1")]);
    assert_eq!(cherry_pick(&g, &m).unwrap(), OpOutcome::Done);
    assert!(r.path().join("b.txt").exists());
}

#[test]
fn cherry_pick_conflict_then_abort() {
    let r = diverged();
    let g = Git::new(r.path());
    assert_eq!(cherry_pick(&g, "topic").unwrap(), OpOutcome::Conflicts);
    abort_op(&g).unwrap();
    assert!(repo_state(&g).unwrap().operation.is_none());
}

#[test]
fn resolve_ours_when_theirs_deleted_file() {
    let r = TestRepo::new();
    r.write("f.txt", "base\n");
    r.commit_all("base");
    r.git(&["checkout", "-b", "topic"]);
    r.git(&["rm", "f.txt"]);
    r.commit_all("delete");
    r.git(&["checkout", "main"]);
    r.write("f.txt", "main edit\n");
    r.commit_all("edit");
    let g = Git::new(r.path());
    assert_eq!(merge(&g, "topic").unwrap(), OpOutcome::Conflicts);
    // Ours kept the file.
    resolve_conflict(&g, "f.txt", Side::Ours).unwrap();
    assert!(conflicted(&g).is_empty());
    assert!(r.path().join("f.txt").exists());
    abort_op(&g).unwrap();

    // Theirs deleted it: choosing theirs removes it.
    assert_eq!(merge(&g, "topic").unwrap(), OpOutcome::Conflicts);
    resolve_conflict(&g, "f.txt", Side::Theirs).unwrap();
    assert!(conflicted(&g).is_empty());
    assert!(!r.path().join("f.txt").exists());
}

#[test]
fn resolve_ours_when_ours_deleted_file() {
    let r = TestRepo::new();
    r.write("f.txt", "base\n");
    r.commit_all("base");
    r.git(&["checkout", "-b", "topic"]);
    r.write("f.txt", "topic edit\n");
    r.commit_all("edit");
    r.git(&["checkout", "main"]);
    r.git(&["rm", "f.txt"]);
    r.commit_all("delete");
    let g = Git::new(r.path());
    assert_eq!(merge(&g, "topic").unwrap(), OpOutcome::Conflicts);
    resolve_conflict(&g, "f.txt", Side::Ours).unwrap();
    assert!(conflicted(&g).is_empty());
    assert!(!r.path().join("f.txt").exists());
}

#[test]
fn continue_with_unresolved_files_returns_conflicts() {
    let r = diverged();
    let g = Git::new(r.path());
    assert_eq!(merge(&g, "topic").unwrap(), OpOutcome::Conflicts);
    assert_eq!(continue_op(&g).unwrap(), OpOutcome::Conflicts);
    mark_resolved(&g, "f.txt").unwrap();
    assert_eq!(continue_op(&g).unwrap(), OpOutcome::Done);
}

#[test]
fn no_operation_errors() {
    let r = diverged();
    let g = Git::new(r.path());
    let e = continue_op(&g).unwrap_err().to_string();
    assert_eq!(e, "No operation in progress");
    assert!(abort_op(&g).is_err());
    merge(&g, "topic").unwrap();
    assert_eq!(
        skip_op(&g).unwrap_err().to_string(),
        "Skip is only available during a rebase"
    );
}

#[test]
fn failure_without_conflicts_propagates() {
    let r = diverged();
    let g = Git::new(r.path());
    assert!(merge(&g, "no-such-branch").is_err());
}

#[test]
fn resolve_on_non_conflicted_dirty_file_errors_and_keeps_edit() {
    let r = TestRepo::new();
    r.write("f.txt", "base\n");
    r.commit_all("base");
    r.write("f.txt", "dirty\n");
    let g = Git::new(r.path());
    for side in [Side::Ours, Side::Theirs] {
        assert!(resolve_conflict(&g, "f.txt", side).is_err());
    }
    assert_eq!(
        std::fs::read_to_string(r.path().join("f.txt")).unwrap(),
        "dirty\n"
    );
}

#[test]
fn resolve_on_unknown_path_errors() {
    let r = diverged();
    let g = Git::new(r.path());
    assert!(resolve_conflict(&g, "nope.txt", Side::Ours).is_err());
}

#[test]
fn resolve_on_directory_prefix_errors_and_keeps_conflict() {
    let r = TestRepo::new();
    r.write("sub/a.txt", "base\n");
    r.commit_all("base");
    r.git(&["checkout", "-b", "topic"]);
    r.write("sub/a.txt", "topic\n");
    r.commit_all("topic");
    r.git(&["checkout", "main"]);
    r.write("sub/a.txt", "main\n");
    r.commit_all("main");
    let g = Git::new(r.path());
    assert_eq!(merge(&g, "topic").unwrap(), OpOutcome::Conflicts);
    assert!(resolve_conflict(&g, "sub", Side::Theirs).is_err());
    assert_eq!(conflicted(&g), vec!["sub/a.txt"]);
}

/// main at "base"; `topic` one commit ahead, tagged `lite` (lightweight) and `ann` (annotated).
fn tagged_ahead() -> TestRepo {
    let r = TestRepo::new();
    r.write("a.txt", "a\n");
    r.commit_all("base");
    r.git(&["checkout", "-b", "topic"]);
    r.write("b.txt", "b\n");
    r.commit_all("b");
    r.git(&["tag", "lite"]);
    r.git(&["tag", "-a", "ann", "-m", "release"]);
    r.git(&["checkout", "main"]);
    r
}

#[test]
fn merge_tag_fast_forwards() {
    // Annotated tags too: git makes a merge commit only for a tag outside refs/tags/.
    for tag in ["lite", "ann"] {
        let r = tagged_ahead();
        let g = Git::new(r.path());
        assert_eq!(merge(&g, tag).unwrap(), OpOutcome::Done, "{tag}");
        assert_eq!(
            r.git(&["rev-parse", "main"]),
            r.git(&["rev-parse", "topic"]),
            "{tag}"
        );
    }
}

#[test]
fn merge_tag_behind_diverged_branch_makes_a_merge_commit_without_an_editor() {
    let r = tagged_ahead();
    r.write("c.txt", "c\n");
    r.commit_all("c");
    let g = Git::new(r.path());
    assert_eq!(merge(&g, "ann").unwrap(), OpOutcome::Done);
    assert_eq!(
        r.git(&["log", "-1", "--format=%P"])
            .split_whitespace()
            .count(),
        2
    );
}

#[test]
fn rebase_onto_tag() {
    let r = tagged_ahead();
    r.write("c.txt", "c\n");
    r.commit_all("c");
    let g = Git::new(r.path());
    assert_eq!(rebase(&g, "ann").unwrap(), OpOutcome::Done);
    assert_eq!(
        r.git(&["rev-parse", "main~1"]),
        r.git(&["rev-parse", "topic"])
    );
}
