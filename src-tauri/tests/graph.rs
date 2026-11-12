mod common;
use common::TestRepo;
use gitsaurus_lib::git::graph::{build_graph, search};
use gitsaurus_lib::git::{Git, RefKind, RefLabel, read_log};

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

fn rows_continuous(rows: &[gitsaurus_lib::git::GraphCommit]) {
    use std::collections::BTreeSet;
    for w in rows.windows(2) {
        let a: BTreeSet<_> = w[0]
            .out
            .iter()
            .chain(&w[0].pass)
            .map(|l| (l.lane, l.color))
            .collect();
        let b: BTreeSet<_> = w[1]
            .into
            .iter()
            .chain(&w[1].pass)
            .map(|l| (l.lane, l.color))
            .collect();
        assert_eq!(a, b, "rows {} -> {}", w[0].index, w[1].index);
    }
}

#[test]
fn build_graph_decorates_refs_and_head_index() {
    let r = TestRepo::new();
    r.write("a.txt", "1\n");
    r.commit_all("c1");
    r.git(&["tag", "v1"]);
    r.write("a.txt", "2\n");
    r.commit_all("c2");
    r.git(&["branch", "feat"]);
    r.write("a.txt", "3\n");
    r.commit_all("c3");
    r.git(&["checkout", "feat"]);
    r.write("b.txt", "f\n");
    r.commit_all("f1");
    r.git(&["checkout", "main"]);
    r.git(&["merge", "--no-ff", "-m", "merge feat", "feat"]);

    let g = build_graph(&Git::new(r.path())).unwrap();
    assert_eq!(g.commits.len(), 5);
    let tip = g.head_index.unwrap() as usize;
    assert_eq!(
        g.commits[tip].refs[0],
        RefLabel {
            kind: RefKind::Local,
            name: "main".into(),
            is_head: true
        }
    );
    let feat = g.commits.iter().find(|c| c.summary == "f1").unwrap();
    assert!(
        feat.refs
            .iter()
            .any(|l| l.kind == RefKind::Local && l.name == "feat" && !l.is_head)
    );
    let first = g.commits.iter().find(|c| c.summary == "c1").unwrap();
    assert!(
        first
            .refs
            .iter()
            .any(|l| l.kind == RefKind::Tag && l.name == "v1")
    );
    assert_eq!(first.short_oid, first.oid[..7]);
    for c in &g.commits {
        assert_eq!(g.by_oid[&c.oid], c.index);
    }
    assert!(g.max_lanes >= 2);
    rows_continuous(&g.commits);
}

#[test]
fn search_matches_summary_author_and_sha_prefix() {
    let r = TestRepo::new();
    r.write("a.txt", "1\n");
    r.commit_all("Add login");
    r.write("a.txt", "2\n");
    r.commit_all("fix LOGIN bug");
    r.write("a.txt", "3\n");
    let docs = r.commit_all("docs");
    let g = build_graph(&Git::new(r.path())).unwrap();
    let idx = |s: &str| g.commits.iter().find(|c| c.summary == s).unwrap().index;
    let mut want = vec![idx("Add login"), idx("fix LOGIN bug")];
    want.sort();
    assert_eq!(search(&g, "login"), want);
    assert_eq!(search(&g, &docs[..6]), vec![idx("docs")]);
    assert!(search(&g, "zzz").is_empty());
    assert!(search(&g, "").is_empty());
}
