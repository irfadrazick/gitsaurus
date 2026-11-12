mod common;
use common::TestRepo;
use gitsaurus_lib::git::{Git, OpKind, parse_porcelain_v2, repo_state};

fn find<'a>(
    files: &'a [gitsaurus_lib::git::FileStatus],
    path: &str,
) -> &'a gitsaurus_lib::git::FileStatus {
    files
        .iter()
        .find(|f| f.path == path)
        .unwrap_or_else(|| panic!("no status entry for {path}"))
}

#[test]
fn status_lists_untracked_modified_staged_renamed() {
    let r = TestRepo::new();
    r.write("mod.txt", "one\n");
    r.write(
        "old.txt",
        "rename me\nwith some content\nto keep similarity\n",
    );
    r.write("keep.txt", "keep\n");
    r.commit_all("init");
    r.write("mod.txt", "two\n");
    r.git(&["mv", "old.txt", "new.txt"]);
    r.write("staged.txt", "s\n");
    r.git(&["add", "staged.txt"]);
    r.write("untracked.txt", "u\n");

    let st = repo_state(&Git::new(r.path())).unwrap();
    assert!(st.operation.is_none());
    assert_eq!(st.head.branch.as_deref(), Some("main"));
    let m = find(&st.files, "mod.txt");
    assert_eq!((m.index.as_str(), m.worktree.as_str()), (".", "M"));
    let s = find(&st.files, "staged.txt");
    assert_eq!((s.index.as_str(), s.worktree.as_str()), ("A", "."));
    let n = find(&st.files, "new.txt");
    assert_eq!((n.index.as_str(), n.worktree.as_str()), ("R", "."));
    assert_eq!(n.orig_path.as_deref(), Some("old.txt"));
    let u = find(&st.files, "untracked.txt");
    assert_eq!((u.index.as_str(), u.worktree.as_str()), (".", "?"));
    assert!(u.untracked && !u.conflicted);
    assert!(st.files.iter().all(|f| f.path != "keep.txt"));
}

#[test]
fn status_in_fresh_repo_lists_untracked() {
    let r = TestRepo::new();
    r.write("a.txt", "a\n");
    r.write("dir/b.txt", "b\n");
    let st = repo_state(&Git::new(r.path())).unwrap();
    assert_eq!(st.head.oid, None);
    assert!(st.operation.is_none());
    assert!(find(&st.files, "a.txt").untracked);
    assert!(find(&st.files, "dir/b.txt").untracked);
}

#[test]
fn merge_conflict_reports_operation_and_conflicted_file() {
    let r = TestRepo::new();
    r.write("f.txt", "base\n");
    r.commit_all("base");
    r.git(&["checkout", "-b", "feat"]);
    r.write("f.txt", "feat\n");
    r.commit_all("feat change");
    r.git(&["checkout", "main"]);
    r.write("f.txt", "main\n");
    r.commit_all("main change");
    r.git_allow_fail(&["merge", "feat"]);

    let st = repo_state(&Git::new(r.path())).unwrap();
    let op = st.operation.expect("operation");
    assert_eq!(op.kind, OpKind::Merge);
    assert_eq!(op.ours_label, "main");
    assert!(op.theirs_label.contains("feat"), "{}", op.theirs_label);
    let f = find(&st.files, "f.txt");
    assert!(f.conflicted);
    assert_eq!((f.index.as_str(), f.worktree.as_str()), ("U", "U"));
}

#[test]
fn rebase_conflict_reports_step_and_total() {
    let r = TestRepo::new();
    r.write("f.txt", "base\n");
    r.commit_all("base");
    r.git(&["checkout", "-b", "feat"]);
    r.write("f.txt", "one\n");
    r.commit_all("c1");
    r.write("g.txt", "g\n");
    r.commit_all("c2");
    r.git(&["checkout", "main"]);
    r.write("f.txt", "main\n");
    r.commit_all("main change");
    r.git(&["checkout", "feat"]);
    r.git_allow_fail(&["rebase", "main"]);

    let st = repo_state(&Git::new(r.path())).unwrap();
    let op = st.operation.expect("operation");
    assert_eq!(op.kind, OpKind::Rebase);
    assert_eq!((op.step, op.total), (Some(1), Some(2)));
    assert_eq!(op.ours_label, "main");
    assert_eq!(op.theirs_label, "feat");
    assert!(find(&st.files, "f.txt").conflicted);
}

#[test]
fn cherry_pick_and_revert_conflicts_report_kind() {
    let r = TestRepo::new();
    r.write("f.txt", "base\n");
    r.commit_all("base");
    r.git(&["checkout", "-b", "feat"]);
    r.write("f.txt", "feat\n");
    let feat = r.commit_all("feat change");
    r.git(&["checkout", "main"]);
    r.write("f.txt", "main\n");
    r.commit_all("main change");
    r.git_allow_fail(&["cherry-pick", &feat]);
    let op = repo_state(&Git::new(r.path())).unwrap().operation.unwrap();
    assert_eq!(op.kind, OpKind::CherryPick);
    assert_eq!(op.theirs_label, feat[..7]);
    r.git(&["cherry-pick", "--abort"]);
    let main_change = r.git(&["rev-parse", "HEAD~0"]).trim().to_string();
    r.write("f.txt", "later\n");
    r.commit_all("later");
    r.git_allow_fail(&["revert", &main_change]);
    let op = repo_state(&Git::new(r.path())).unwrap().operation.unwrap();
    assert_eq!(op.kind, OpKind::Revert);
    assert_eq!(op.theirs_label, format!("revert {}", &main_change[..7]));
}

#[test]
fn parse_porcelain_v2_literal() {
    let raw = "1 .M N... 100644 100644 100644 aaa bbb dir with space/ünï cödé.txt\0\
2 R. N... 100644 100644 100644 aaa bbb R100 new name.txt\0old name.txt\0\
u UU N... 100644 100644 100644 100644 a b c conflict.txt\0\
? untracked file.txt\0\
! ignored.txt\0\
1 A. N... 000000 100644 100644 000 bbb added.txt\0";
    let v = parse_porcelain_v2(raw.as_bytes());
    assert_eq!(v.len(), 5);
    assert_eq!(v[0].path, "dir with space/ünï cödé.txt");
    assert_eq!((v[0].index.as_str(), v[0].worktree.as_str()), (".", "M"));
    assert_eq!(v[1].path, "new name.txt");
    assert_eq!(v[1].orig_path.as_deref(), Some("old name.txt"));
    assert_eq!(v[1].index, "R");
    assert!(v[2].conflicted);
    assert_eq!((v[2].index.as_str(), v[2].worktree.as_str()), ("U", "U"));
    assert_eq!(v[3].path, "untracked file.txt");
    assert!(v[3].untracked);
    assert_eq!((v[3].index.as_str(), v[3].worktree.as_str()), (".", "?"));
    assert_eq!(v[4].path, "added.txt");
}
