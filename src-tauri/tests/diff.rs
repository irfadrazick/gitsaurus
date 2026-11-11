mod common;
use common::TestRepo;
use gitsaurus_lib::git::{
    Change, DiffSource, Git, LineKind, commit_details, file_diff, parse_unified,
};

#[test]
fn parse_unified_numbers_lines() {
    let d = parse_unified(
        "f.txt",
        None,
        "diff --git a/f.txt b/f.txt\n--- a/f.txt\n+++ b/f.txt\n@@ -1,3 +1,3 @@ ctx\n a\n-b\n+B\n c\n\\ No newline at end of file\n",
    );
    let l = &d.hunks[0].lines;
    assert_eq!(
        (l[1].kind, l[1].old_no, l[1].new_no),
        (LineKind::Del, Some(2), None)
    );
    assert_eq!((l[2].kind, l[2].new_no), (LineKind::Add, Some(2)));
    assert_eq!(l[4].kind, LineKind::NoNewline);
    assert_eq!((l[4].old_no, l[4].new_no), (None, None));
    assert_eq!(l[0].text, "a");
    let h = &d.hunks[0];
    assert_eq!(
        (h.old_start, h.old_lines, h.new_start, h.new_lines),
        (1, 3, 1, 3)
    );
    assert_eq!(h.header, "@@ -1,3 +1,3 @@ ctx");
}

#[test]
fn parse_unified_header_without_counts() {
    let d = parse_unified("f", None, "@@ -0,0 +1 @@\n+x\n");
    let h = &d.hunks[0];
    assert_eq!(
        (h.old_start, h.old_lines, h.new_start, h.new_lines),
        (0, 0, 1, 1)
    );
}

#[test]
fn binary_file_is_flagged() {
    let r = TestRepo::new();
    r.write("a.txt", "a\n");
    r.commit_all("init");
    std::fs::write(r.path().join("b.bin"), [0u8, 1, 2, 0, 255]).unwrap();
    let oid = r.commit_all("bin");
    let d = file_diff(
        &Git::new(r.path()),
        "b.bin",
        None,
        &DiffSource::Commit { oid },
        3,
    )
    .unwrap();
    assert!(d.binary && d.hunks.is_empty());
}

#[test]
fn too_large_is_flagged() {
    let r = TestRepo::new();
    let big: String = (0..25_000).map(|i| format!("line {i}\n")).collect();
    r.write("big.txt", &big);
    let oid = r.commit_all("big");
    let d = file_diff(
        &Git::new(r.path()),
        "big.txt",
        None,
        &DiffSource::Commit { oid },
        3,
    )
    .unwrap();
    assert!(d.too_large && d.hunks.is_empty());
}

#[test]
fn commit_details_lists_files_coauthors_and_rename() {
    let r = TestRepo::new();
    r.write(
        "a.txt",
        "some content\nthat is long enough\nto be a rename\n",
    );
    r.commit_all("init");
    r.git(&["mv", "a.txt", "b.txt"]);
    r.write("c.txt", "c\n");
    let oid = r.commit_all(
        "subject line\n\nx\n\nCo-authored-by: Claude <noreply@anthropic.com>\nco-authored-by: Bob B <bob@x.io>",
    );
    let d = commit_details(&Git::new(r.path()), &oid).unwrap();
    assert_eq!(d.oid, oid);
    assert_eq!(d.summary, "subject line");
    assert_eq!(d.author.name, "Test User");
    assert_eq!(d.author.email, "test@example.com");
    assert!(d.author.time > 0);
    assert_eq!(d.parents.len(), 1);
    assert_eq!(d.co_authors.len(), 2);
    assert_eq!(d.co_authors[0].email, "noreply@anthropic.com");
    assert_eq!(d.co_authors[0].name, "Claude");
    assert_eq!(d.co_authors[0].time, 0);
    assert_eq!(d.co_authors[1].name, "Bob B");
    assert!(d.body.starts_with("x"));
    let ren = d.files.iter().find(|f| f.path == "b.txt").unwrap();
    assert_eq!(ren.change, Change::Renamed);
    assert_eq!(ren.orig_path.as_deref(), Some("a.txt"));
    let add = d.files.iter().find(|f| f.path == "c.txt").unwrap();
    assert_eq!(add.change, Change::Added);
    assert_eq!(d.files.len(), 2);

    // Rename diff pairs the two paths.
    let fd = file_diff(
        &Git::new(r.path()),
        "b.txt",
        Some("a.txt"),
        &DiffSource::Commit { oid },
        3,
    )
    .unwrap();
    assert!(fd.hunks.is_empty() && !fd.binary);
    assert_eq!(fd.orig_path.as_deref(), Some("a.txt"));
}

#[test]
fn root_and_merge_commit_details_work() {
    let r = TestRepo::new();
    r.write("a.txt", "a\n");
    let root = r.commit_all("root");
    let d = commit_details(&Git::new(r.path()), &root).unwrap();
    assert!(d.parents.is_empty());
    assert_eq!(d.files.len(), 1);
    assert_eq!(d.files[0].change, Change::Added);

    r.git(&["checkout", "-b", "feat"]);
    r.write("f.txt", "f\n");
    r.commit_all("feat");
    r.git(&["checkout", "main"]);
    r.write("m.txt", "m\n");
    r.commit_all("main");
    r.git(&["merge", "--no-ff", "-m", "merge", "feat"]);
    let mc = r.git(&["rev-parse", "HEAD"]).trim().to_string();
    let d = commit_details(&Git::new(r.path()), &mc).unwrap();
    assert_eq!(d.parents.len(), 2);
    // Compared to the first parent (main): only feat's file appears.
    assert_eq!(d.files.len(), 1);
    assert_eq!(d.files[0].path, "f.txt");

    let fd = file_diff(
        &Git::new(r.path()),
        "f.txt",
        None,
        &DiffSource::Commit { oid: mc },
        3,
    )
    .unwrap();
    assert_eq!(fd.hunks.len(), 1);
    let root_fd = file_diff(
        &Git::new(r.path()),
        "a.txt",
        None,
        &DiffSource::Commit { oid: root },
        3,
    )
    .unwrap();
    assert_eq!(root_fd.hunks[0].lines[0].kind, LineKind::Add);
}

#[test]
fn staged_unstaged_untracked_and_full_file_diffs() {
    let r = TestRepo::new();
    let fifty: String = (1..=50).map(|i| format!("l{i}\n")).collect();
    r.write("f.txt", &fifty);
    r.commit_all("init");
    let g = Git::new(r.path());

    r.write("f.txt", &fifty.replace("l10\n", "L10\n"));
    r.git(&["add", "f.txt"]);
    r.write(
        "f.txt",
        &fifty.replace("l10\n", "L10\n").replace("l40\n", "L40\n"),
    );

    let staged = file_diff(&g, "f.txt", None, &DiffSource::Staged, 3).unwrap();
    let unstaged = file_diff(&g, "f.txt", None, &DiffSource::Unstaged, 3).unwrap();
    assert_ne!(staged, unstaged);
    let adds = |d: &gitsaurus_lib::git::FileDiff| -> Vec<String> {
        d.hunks
            .iter()
            .flat_map(|h| h.lines.iter())
            .filter(|l| l.kind == LineKind::Add)
            .map(|l| l.text.clone())
            .collect()
    };
    assert_eq!(adds(&staged), vec!["L10"]);
    assert_eq!(adds(&unstaged), vec!["L40"]);

    r.write("new.txt", "x\ny\nz\n");
    let u = file_diff(&g, "new.txt", None, &DiffSource::Untracked, 3).unwrap();
    assert_eq!(u.hunks.len(), 1);
    assert_eq!(u.hunks[0].lines.len(), 3);
    assert!(u.hunks[0].lines.iter().all(|l| l.kind == LineKind::Add));

    let full = file_diff(&g, "f.txt", None, &DiffSource::Unstaged, 1_000_000).unwrap();
    assert_eq!(full.hunks.len(), 1);
    let new_side = full.hunks[0]
        .lines
        .iter()
        .filter(|l| l.new_no.is_some())
        .count();
    assert_eq!(new_side, 50);
}

#[test]
fn working_file_emits_numbered_context_lines_and_flags_binary() {
    let r = TestRepo::new();
    r.write("c.txt", "<<<<<<< ours\na\n=======\nb\n>>>>>>> theirs\n");
    std::fs::write(r.path().join("b.bin"), [1u8, 0, 2]).unwrap();
    let g = Git::new(r.path());
    let d = file_diff(&g, "c.txt", None, &DiffSource::WorkingFile, 1_000_000).unwrap();
    assert_eq!(d.hunks.len(), 1);
    let l = &d.hunks[0].lines;
    assert_eq!(l.len(), 5);
    assert_eq!((l[4].old_no, l[4].new_no), (Some(5), Some(5)));
    assert!(l.iter().all(|x| x.kind == LineKind::Context));
    let b = file_diff(&g, "b.bin", None, &DiffSource::WorkingFile, 3).unwrap();
    assert!(b.binary && b.hunks.is_empty());
}

#[test]
fn working_file_ignores_context_and_returns_whole_file() {
    let r = TestRepo::new();
    let body: String = (1..=25).map(|i| format!("row{i}\n")).collect();
    r.write("w.txt", &body);
    let d = file_diff(
        &Git::new(r.path()),
        "w.txt",
        None,
        &DiffSource::WorkingFile,
        3,
    )
    .unwrap();
    assert_eq!(d.hunks.len(), 1);
    let h = &d.hunks[0];
    assert_eq!((h.old_lines, h.new_lines), (25, 25));
    assert_eq!(h.lines.len(), 25);
    for (i, l) in h.lines.iter().enumerate() {
        let n = i as u32 + 1;
        assert_eq!((l.old_no, l.new_no), (Some(n), Some(n)));
        assert_eq!(l.text, format!("row{n}"));
    }
}
