mod common;
use common::TestRepo;
use gitsaurus_lib::git::{Git, head, list_refs, parse_track};

#[test]
fn parse_track_variants() {
    assert_eq!(parse_track("[ahead 2, behind 3]"), (2, 3, false));
    assert_eq!(parse_track("[behind 1]"), (0, 1, false));
    assert_eq!(parse_track("[ahead 4]"), (4, 0, false));
    assert_eq!(parse_track("[gone]"), (0, 0, true));
    assert_eq!(parse_track(""), (0, 0, false));
}

#[test]
fn lists_local_remote_tags_stash_with_upstream_counts() {
    let origin = TestRepo::bare();
    let seed = TestRepo::new();
    seed.write("a.txt", "a\n");
    seed.commit_all("init");
    seed.git(&["remote", "add", "origin", origin.path().to_str().unwrap()]);
    seed.git(&["push", "origin", "main"]);
    let c = TestRepo::clone_of(&origin);
    c.write("b.txt", "b\n");
    let local_oid = c.commit_all("local");
    c.git(&["branch", "feat"]);
    c.git(&["tag", "v1"]);
    c.git(&["tag", "-a", "v2", "-m", "annotated"]);
    c.write("a.txt", "changed\n");
    c.git(&["stash", "push", "-m", "wip"]);

    let r = list_refs(&Git::new(c.path())).unwrap();
    assert_eq!(r.head.branch.as_deref(), Some("main"));
    assert_eq!(r.local.len(), 2);
    let main = &r.local[0];
    assert_eq!(main.name, "main");
    assert_eq!(main.full_ref, "refs/heads/main");
    assert!(main.is_head);
    assert_eq!(main.upstream.as_deref(), Some("origin/main"));
    assert_eq!((main.ahead, main.behind, main.gone), (1, 0, false));
    assert_eq!(main.oid, local_oid);
    let feat = &r.local[1];
    assert_eq!(feat.name, "feat");
    assert!(!feat.is_head);
    assert_eq!(feat.upstream, None);

    assert_eq!(r.remote.len(), 1);
    assert_eq!(r.remote[0].full_name, "origin/main");
    assert_eq!(r.remote[0].name, "main");
    assert_eq!(r.remote[0].remote, "origin");
    assert_eq!(r.remotes, vec!["origin".to_string()]);

    assert_eq!(r.tags.len(), 2);
    assert_eq!(r.tags[0].name, "v1");
    assert!(!r.tags[0].annotated);
    assert_eq!(r.tags[1].name, "v2");
    assert!(r.tags[1].annotated);
    assert_eq!(r.tags[1].oid, local_oid);

    assert_eq!(r.stashes.len(), 1);
    assert_eq!(r.stashes[0].index, 0);
    assert!(r.stashes[0].message.contains("wip"));
}

#[test]
fn remote_names_with_slashes_split_on_known_remote() {
    let origin = TestRepo::bare();
    let seed = TestRepo::new();
    seed.write("a.txt", "a\n");
    seed.commit_all("init");
    seed.git(&["branch", "feature/x"]);
    seed.git(&["remote", "add", "origin", origin.path().to_str().unwrap()]);
    seed.git(&["push", "origin", "main", "feature/x"]);
    seed.git(&["fetch", "origin"]);
    let r = list_refs(&Git::new(seed.path())).unwrap();
    let x = r
        .remote
        .iter()
        .find(|b| b.full_name == "origin/feature/x")
        .unwrap();
    assert_eq!(
        (x.remote.as_str(), x.name.as_str()),
        ("origin", "feature/x")
    );
}

#[test]
fn detached_head_has_no_branch() {
    let r = TestRepo::new();
    r.write("a.txt", "a\n");
    let oid = r.commit_all("c1");
    r.git(&["tag", "v1"]);
    r.write("a.txt", "b\n");
    r.commit_all("c2");
    r.git(&["checkout", "--detach", "v1"]);
    let h = head(&Git::new(r.path())).unwrap();
    assert_eq!(h.branch, None);
    assert_eq!(h.oid, Some(oid));
}

#[test]
fn unborn_head_has_no_oid() {
    let r = TestRepo::new();
    let h = head(&Git::new(r.path())).unwrap();
    assert_eq!(h.branch.as_deref(), Some("main"));
    assert_eq!(h.oid, None);
}

#[test]
fn list_refs_on_empty_repo_is_empty() {
    let r = TestRepo::new();
    let refs = list_refs(&Git::new(r.path())).unwrap();
    assert!(refs.local.is_empty() && refs.remote.is_empty() && refs.tags.is_empty());
    assert!(refs.stashes.is_empty());
    assert_eq!(refs.head.branch.as_deref(), Some("main"));
    assert_eq!(refs.head.oid, None);
}
