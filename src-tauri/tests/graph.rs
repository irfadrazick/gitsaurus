mod common;
use common::TestRepo;
use gitsaurus_lib::git::{Git, read_log};

#[test]
fn read_log_on_unborn_repo_is_empty() {
    let r = TestRepo::new();
    assert!(read_log(&Git::new(r.path())).unwrap().is_empty());
}

#[test]
fn read_log_excludes_stash_commits() {
    let r = TestRepo::new();
    r.write("a.txt", "a\n");
    r.commit_all("init");
    r.write("a.txt", "changed\n");
    r.git(&["stash", "push", "-m", "wip"]);
    let v = read_log(&Git::new(r.path())).unwrap();
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].summary, "init");
    assert!(
        v.iter()
            .all(|c| c.summary != "WIP on main" && c.summary != "index on main")
    );
}

#[test]
fn read_log_reads_fields_and_parents() {
    let r = TestRepo::new();
    r.write("a.txt", "a\n");
    let first = r.commit_all("first");
    r.write("a.txt", "b\n");
    r.commit_all("second\n\nbody line");
    let v = read_log(&Git::new(r.path())).unwrap();
    assert_eq!(v.len(), 2);
    assert_eq!(v[0].summary, "second");
    assert!(v[0].body.contains("body line"));
    assert_eq!(v[0].parents, vec![first]);
}
