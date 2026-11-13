mod common;
use common::TestRepo;
use gitsaurus_lib::git::ops::branch::*;
use gitsaurus_lib::git::{
    Git, GitError, GitResult, OpOutcome, RefKind, ResetMode, list_refs, repo_state,
};

fn setup() -> (TestRepo, TestRepo) {
    let bare = TestRepo::bare();
    let a = TestRepo::clone_of(&bare);
    a.write("f.txt", "1\n");
    a.commit_all("c1");
    a.git(&["push", "-u", "origin", "main"]);
    (bare, a)
}

fn stash_oid(r: &TestRepo, i: u32) -> String {
    r.git(&["rev-parse", &format!("stash@{{{i}}}")])
        .trim()
        .to_string()
}

fn branch_names(g: &Git) -> Vec<String> {
    list_refs(g)
        .unwrap()
        .local
        .into_iter()
        .map(|b| b.name)
        .collect()
}

#[test]
fn checkout_remote_branch_creates_tracking_local() {
    let (bare, a) = setup();
    a.git(&["push", "origin", "main:x"]);
    let b = TestRepo::clone_of(&bare);
    let g = Git::new(b.path());
    checkout(&g, "origin/x", RefKind::Remote).unwrap();
    let br = list_refs(&g)
        .unwrap()
        .local
        .into_iter()
        .find(|b| b.name == "x")
        .unwrap();
    assert!(br.is_head);
    assert_eq!(br.upstream.as_deref(), Some("origin/x"));
    // existing local: plain checkout
    checkout(&g, "main", RefKind::Local).unwrap();
    checkout(&g, "origin/x", RefKind::Remote).unwrap();
    assert!(
        list_refs(&g)
            .unwrap()
            .local
            .iter()
            .any(|b| b.name == "x" && b.is_head)
    );
}

#[test]
fn checkout_tag_detaches() {
    let (_b, a) = setup();
    a.git(&["tag", "v1"]);
    let g = Git::new(a.path());
    checkout(&g, "v1", RefKind::Tag).unwrap();
    assert!(gitsaurus_lib::git::head(&g).unwrap().branch.is_none());
    checkout(&g, "main", RefKind::Local).unwrap();
    let oid = a.git(&["rev-parse", "HEAD"]);
    checkout_commit(&g, oid.trim()).unwrap();
    assert!(gitsaurus_lib::git::head(&g).unwrap().branch.is_none());
    assert!(matches!(
        checkout_commit(&g, "--orphan"),
        Err(GitError::Failed { .. })
    ));
}

#[test]
fn create_rename_delete_branch() {
    let (_b, a) = setup();
    let g = Git::new(a.path());
    create_branch(&g, "feat", "HEAD", true).unwrap();
    a.write("g.txt", "x");
    a.commit_all("on feat");
    checkout(&g, "main", RefKind::Local).unwrap();
    create_branch(&g, "other", "main", false).unwrap();
    rename_branch(&g, "other", "other2").unwrap();
    assert!(branch_names(&g).contains(&"other2".to_string()));
    delete_branch(&g, "other2", false, None).unwrap();
    assert!(matches!(
        delete_branch(&g, "feat", false, None),
        Err(GitError::Failed { .. })
    ));
    delete_branch(&g, "feat", true, None).unwrap();
    assert_eq!(branch_names(&g), vec!["main"]);
}

#[test]
fn delete_branch_also_on_remote() {
    let (bare, a) = setup();
    a.git(&["push", "origin", "main:x"]);
    let g = Git::new(a.path());
    create_branch(&g, "x", "main", false).unwrap();
    delete_branch(&g, "x", false, Some("origin")).unwrap();
    assert!(bare.git(&["for-each-ref", "refs/heads/x"]).is_empty());
    // failed local delete leaves the remote alone
    a.git(&["push", "origin", "main:y"]);
    create_branch(&g, "y", "HEAD", true).unwrap();
    assert!(delete_branch(&g, "y", false, Some("origin")).is_err());
    assert!(!bare.git(&["for-each-ref", "refs/heads/y"]).is_empty());
}

#[test]
fn delete_remote_branch_removes_it_there_and_keeps_the_local_one() {
    let (bare, a) = setup();
    let g = Git::new(a.path());
    create_branch(&g, "feat/x", "main", false).unwrap();
    a.git(&["push", "-u", "origin", "feat/x"]);
    delete_remote_branch(&g, "origin/feat/x").unwrap();
    assert!(bare.git(&["for-each-ref", "refs/heads/feat/x"]).is_empty());
    // The remote-tracking ref goes too; the local branch stays and shows as gone.
    assert!(
        a.git(&["for-each-ref", "refs/remotes/origin/feat/x"])
            .is_empty()
    );
    let refs = list_refs(&g).unwrap();
    let local = refs.local.iter().find(|b| b.name == "feat/x").unwrap();
    assert!(local.gone);
}

#[test]
fn delete_remote_branch_refuses_unknown_remotes_and_options() {
    let (_bare, a) = setup();
    let g = Git::new(a.path());
    assert!(delete_remote_branch(&g, "nowhere/main").is_err());
    assert!(delete_remote_branch(&g, "--all").is_err());
    assert!(delete_remote_branch(&g, "origin/-x").is_err());
}

#[test]
fn invalid_branch_name_rejected() {
    let (_b, a) = setup();
    let g = Git::new(a.path());
    match create_branch(&g, "bad..name", "HEAD", false) {
        Err(GitError::Failed { message, .. }) => {
            assert_eq!(message, "Invalid branch name: bad..name")
        }
        o => panic!("{o:?}"),
    }
    assert!(validate_ref_name(&g, "-x").is_err());
    assert!(validate_ref_name(&g, "ok/name").is_ok());
    assert!(rename_branch(&g, "main", "a b").is_err());
    assert!(create_branch(&g, "ok", "--foo", false).is_err());
    assert!(validate_tag_name(&g, "bad..tag").is_err());
}

#[test]
fn create_annotated_and_lightweight_tags_and_delete() {
    let (bare, a) = setup();
    let g = Git::new(a.path());
    create_tag(&g, "light", "HEAD", None).unwrap();
    create_tag(&g, "blank", "HEAD", Some("  ")).unwrap();
    create_tag(&g, "ann", "HEAD", Some("hello")).unwrap();
    let tags = list_refs(&g).unwrap().tags;
    let t = |n: &str| tags.iter().find(|t| t.name == n).unwrap().annotated;
    assert!(!t("light") && !t("blank") && t("ann"));
    a.git(&["push", "origin", "ann"]);
    delete_tag(&g, "ann", Some("origin")).unwrap();
    delete_tag(&g, "light", None).unwrap();
    assert!(bare.git(&["tag"]).is_empty());
    assert!(
        !list_refs(&g)
            .unwrap()
            .tags
            .iter()
            .any(|t| t.name == "ann" || t.name == "light")
    );
    assert!(create_tag(&g, "-x", "HEAD", None).is_err());
}

#[test]
fn reset_soft_mixed_hard() {
    let (_b, a) = setup();
    let g = Git::new(a.path());
    let first = a.git(&["rev-parse", "HEAD"]).trim().to_string();
    a.write("f.txt", "2\n");
    a.commit_all("c2");
    reset(&g, &first, ResetMode::Soft).unwrap();
    let f = &repo_state(&g).unwrap().files[0];
    assert_eq!((f.index.as_str(), f.worktree.as_str()), ("M", "."));
    a.git(&["commit", "-am", "c2"]);
    reset(&g, &first, ResetMode::Mixed).unwrap();
    let f = &repo_state(&g).unwrap().files[0];
    assert_eq!((f.index.as_str(), f.worktree.as_str()), (".", "M"));
    reset(&g, &first, ResetMode::Hard).unwrap();
    assert!(repo_state(&g).unwrap().files.is_empty());
    assert!(matches!(
        reset(&g, "--hard", ResetMode::Hard),
        Err(GitError::Failed { .. })
    ));
}

#[test]
fn stash_push_pop_apply_drop() {
    let (_b, a) = setup();
    let g = Git::new(a.path());
    a.write("f.txt", "changed\n");
    a.write("new.txt", "untracked\n");
    stash_push(&g, Some("wip")).unwrap();
    assert!(repo_state(&g).unwrap().files.is_empty());
    assert!(a.git(&["stash", "list"]).contains("wip"));
    assert_eq!(
        stash_apply(&g, 0, &stash_oid(&a, 0)).unwrap(),
        OpOutcome::Done
    );
    assert_eq!(repo_state(&g).unwrap().files.len(), 2);
    a.git(&["checkout", "--", "f.txt"]);
    std::fs::remove_file(a.path().join("new.txt")).unwrap();
    assert_eq!(
        stash_pop(&g, 0, &stash_oid(&a, 0)).unwrap(),
        OpOutcome::Done
    );
    assert_eq!(repo_state(&g).unwrap().files.len(), 2);
    assert!(a.git(&["stash", "list"]).is_empty());
    stash_push(&g, None).unwrap();
    a.write("f.txt", "again\n");
    stash_push(&g, Some("  ")).unwrap();
    assert_eq!(a.git(&["stash", "list"]).lines().count(), 2);
    stash_drop(&g, 1, &stash_oid(&a, 1)).unwrap();
    assert_eq!(a.git(&["stash", "list"]).lines().count(), 1);
}

#[test]
fn stash_pop_that_conflicts_returns_conflicts_and_keeps_the_stash() {
    let (_b, a) = setup();
    let g = Git::new(a.path());
    a.write("f.txt", "stashed\n");
    stash_push(&g, Some("wip")).unwrap();
    a.write("f.txt", "committed\n");
    a.commit_all("c2");
    assert_eq!(
        stash_pop(&g, 0, &stash_oid(&a, 0)).unwrap(),
        OpOutcome::Conflicts
    );
    let st = repo_state(&g).unwrap();
    assert!(st.files.iter().any(|f| f.path == "f.txt" && f.conflicted));
    assert!(st.operation.is_none());
    assert_eq!(a.git(&["stash", "list"]).lines().count(), 1);
}

#[test]
fn stash_apply_that_conflicts_returns_conflicts() {
    let (_b, a) = setup();
    let g = Git::new(a.path());
    a.write("f.txt", "stashed\n");
    stash_push(&g, None).unwrap();
    a.write("f.txt", "committed\n");
    a.commit_all("c2");
    assert_eq!(
        stash_apply(&g, 0, &stash_oid(&a, 0)).unwrap(),
        OpOutcome::Conflicts
    );
}

#[test]
fn stash_apply_refused_by_local_changes_is_an_error() {
    let (_b, a) = setup();
    let g = Git::new(a.path());
    a.write("f.txt", "stashed\n");
    stash_push(&g, None).unwrap();
    a.write("f.txt", "local edit\n");
    assert!(matches!(
        stash_apply(&g, 0, &stash_oid(&a, 0)),
        Err(GitError::Failed { .. })
    ));
    assert!(repo_state(&g).unwrap().files.iter().all(|f| !f.conflicted));
}

#[test]
fn stash_ops_refuse_when_the_index_points_at_another_stash() {
    let (_b, a) = setup();
    let g = Git::new(a.path());
    a.write("f.txt", "first\n");
    stash_push(&g, Some("first")).unwrap();
    let first = stash_oid(&a, 0);
    a.write("f.txt", "second\n");
    stash_push(&g, Some("second")).unwrap();
    // The user picked "first" at index 0, but a new stash has since moved it to index 1.
    let changed = |r: GitResult<OpOutcome>| match r {
        Err(GitError::Failed { message, .. }) => message == "The stash list changed; please retry",
        _ => false,
    };
    assert!(changed(stash_apply(&g, 0, &first)));
    assert!(changed(stash_pop(&g, 0, &first)));
    assert!(matches!(
        stash_drop(&g, 0, &first),
        Err(GitError::Failed { message, .. }) if message == "The stash list changed; please retry"
    ));
    assert!(changed(stash_pop(&g, 5, &first)));
    assert_eq!(a.git(&["stash", "list"]).lines().count(), 2);
    assert!(repo_state(&g).unwrap().files.is_empty());
    // The matching index works.
    stash_drop(&g, 1, &first).unwrap();
    assert!(a.git(&["stash", "list"]).contains("second"));
    assert!(!a.git(&["stash", "list"]).contains("first"));
}
