mod common;
use common::TestRepo;
use gitsaurus_lib::git::{Git, list_refs, list_worktrees, prune_worktrees, remove_worktree};
use std::path::{Path, PathBuf};

fn canon(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap()
}

/// A repo with one commit plus a linked worktree `wt` on branch `feat`, inside a shared temp dir.
fn setup() -> (TestRepo, tempfile::TempDir, PathBuf) {
    let r = TestRepo::new();
    r.write("a.txt", "a\n");
    r.commit_all("init");
    let dir = tempfile::tempdir().unwrap();
    let wt = dir.path().join("wt");
    r.git(&["worktree", "add", "-b", "feat", wt.to_str().unwrap()]);
    (r, dir, canon(&wt))
}

#[test]
fn lists_main_and_linked_worktrees_from_either_side() {
    let (r, _dir, wt) = setup();
    let from_main = list_worktrees(&Git::new(r.path())).unwrap();
    assert_eq!(from_main.len(), 2);
    assert!(from_main[0].is_main && from_main[0].is_current);
    assert_eq!(from_main[0].branch.as_deref(), Some("main"));
    assert_eq!(from_main[1].branch.as_deref(), Some("feat"));
    assert_eq!(from_main[1].name, "wt");
    assert!(!from_main[1].is_main && !from_main[1].is_current && !from_main[1].missing);

    let from_wt = list_worktrees(&Git::new(&wt)).unwrap();
    assert!(!from_wt[0].is_current);
    assert!(from_wt[1].is_current);

    // list_refs carries them too.
    assert_eq!(list_refs(&Git::new(r.path())).unwrap().worktrees.len(), 2);
}

#[test]
fn a_deleted_folder_is_missing_until_pruned() {
    let (r, _dir, wt) = setup();
    std::fs::remove_dir_all(&wt).unwrap();
    let g = Git::new(r.path());
    assert!(list_worktrees(&g).unwrap()[1].missing);
    prune_worktrees(&g).unwrap();
    assert_eq!(list_worktrees(&g).unwrap().len(), 1);
}

#[test]
fn removes_a_clean_worktree_and_keeps_its_branch() {
    let (r, _dir, wt) = setup();
    let g = Git::new(r.path());
    remove_worktree(&g, wt.to_str().unwrap(), false).unwrap();
    assert!(!wt.exists());
    assert_eq!(list_worktrees(&g).unwrap().len(), 1);
    assert!(r.git(&["branch", "--list", "feat"]).contains("feat"));
}

#[test]
fn a_dirty_worktree_needs_force() {
    let (r, _dir, wt) = setup();
    std::fs::write(wt.join("new.txt"), "x").unwrap();
    let g = Git::new(r.path());
    let err = format!(
        "{:?}",
        remove_worktree(&g, wt.to_str().unwrap(), false).unwrap_err()
    );
    assert!(err.contains("modified or untracked files"), "{err}");
    assert!(wt.exists());
    remove_worktree(&g, wt.to_str().unwrap(), true).unwrap();
    assert!(!wt.exists());
}

#[test]
fn refuses_the_main_and_the_current_worktree() {
    let (r, _dir, wt) = setup();
    let main = canon(r.path());
    let from_main = Git::new(r.path());
    assert!(remove_worktree(&from_main, main.to_str().unwrap(), true).is_err());
    let from_wt = Git::new(&wt);
    assert!(remove_worktree(&from_wt, wt.to_str().unwrap(), true).is_err());
    assert!(wt.exists());
    assert!(remove_worktree(&from_main, "/not/a/worktree", true).is_err());
}
