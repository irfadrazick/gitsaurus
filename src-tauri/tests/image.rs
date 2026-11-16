mod common;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use common::TestRepo;
use gitsaurus_lib::git::{DiffSource, Git, ImageVersion, image::MAX_IMAGE_BYTES, image_versions};

/// Binary contents (with NUL bytes, like a real PNG) that differ per `tag`.
fn img(tag: u8) -> Vec<u8> {
    vec![0x89, b'P', b'N', b'G', 0, 0, tag, 0, 1, 2, 3]
}

fn put(r: &TestRepo, rel: &str, bytes: &[u8]) {
    std::fs::write(r.path().join(rel), bytes).unwrap();
}

fn bytes(v: &Option<ImageVersion>) -> Option<Vec<u8>> {
    v.as_ref()
        .map(|v| STANDARD.decode(v.data.as_ref().expect("data")).unwrap())
}

fn commit(oid: &str) -> DiffSource {
    DiffSource::Commit { oid: oid.into() }
}

#[test]
fn commit_shows_parent_and_commit_versions() {
    let r = TestRepo::new();
    put(&r, "a.png", &img(1));
    let first = r.commit_all("add");
    put(&r, "a.png", &img(2));
    let second = r.commit_all("change");
    let g = Git::new(r.path());

    let v = image_versions(&g, "a.png", None, &commit(&second)).unwrap();
    assert_eq!(bytes(&v.old), Some(img(1)));
    assert_eq!(bytes(&v.new), Some(img(2)));
    assert_eq!(v.new.unwrap().size, img(2).len() as u64);

    // Added in the root commit: nothing before it.
    let v = image_versions(&g, "a.png", None, &commit(&first)).unwrap();
    assert_eq!((bytes(&v.old), bytes(&v.new)), (None, Some(img(1))));
}

#[test]
fn deleted_and_renamed_in_a_commit() {
    let r = TestRepo::new();
    put(&r, "a.png", &img(1));
    put(&r, "b.png", &img(3));
    r.commit_all("add");
    r.git(&["mv", "a.png", "moved.png"]);
    r.git(&["rm", "-q", "b.png"]);
    let c = r.commit_all("move and delete");
    let g = Git::new(r.path());

    let v = image_versions(&g, "b.png", None, &commit(&c)).unwrap();
    assert_eq!((bytes(&v.old), bytes(&v.new)), (Some(img(3)), None));

    let v = image_versions(&g, "moved.png", Some("a.png"), &commit(&c)).unwrap();
    assert_eq!((bytes(&v.old), bytes(&v.new)), (Some(img(1)), Some(img(1))));
}

#[test]
fn staged_unstaged_and_untracked() {
    let r = TestRepo::new();
    put(&r, "a.png", &img(1));
    r.commit_all("add");
    put(&r, "a.png", &img(2));
    r.git(&["add", "a.png"]);
    put(&r, "a.png", &img(3));
    put(&r, "new.png", &img(4));
    let g = Git::new(r.path());

    let v = image_versions(&g, "a.png", None, &DiffSource::Staged).unwrap();
    assert_eq!((bytes(&v.old), bytes(&v.new)), (Some(img(1)), Some(img(2))));

    let v = image_versions(&g, "a.png", None, &DiffSource::Unstaged).unwrap();
    assert_eq!((bytes(&v.old), bytes(&v.new)), (Some(img(2)), Some(img(3))));

    let v = image_versions(&g, "new.png", None, &DiffSource::Untracked).unwrap();
    assert_eq!((bytes(&v.old), bytes(&v.new)), (None, Some(img(4))));

    let v = image_versions(&g, "a.png", None, &DiffSource::WorkingFile).unwrap();
    assert_eq!((bytes(&v.old), bytes(&v.new)), (None, Some(img(3))));
}

#[test]
fn unstaged_deletion_has_no_new_version() {
    let r = TestRepo::new();
    put(&r, "a.png", &img(1));
    r.commit_all("add");
    std::fs::remove_file(r.path().join("a.png")).unwrap();
    let g = Git::new(r.path());
    let v = image_versions(&g, "a.png", None, &DiffSource::Unstaged).unwrap();
    assert_eq!((bytes(&v.old), bytes(&v.new)), (Some(img(1)), None));
}

#[test]
fn staged_in_a_repo_without_commits() {
    let r = TestRepo::new();
    put(&r, "a.png", &img(1));
    r.git(&["add", "a.png"]);
    let g = Git::new(r.path());
    let v = image_versions(&g, "a.png", None, &DiffSource::Staged).unwrap();
    assert_eq!((bytes(&v.old), bytes(&v.new)), (None, Some(img(1))));
}

#[test]
fn oversized_versions_report_size_only() {
    let r = TestRepo::new();
    let big = MAX_IMAGE_BYTES + 1;
    std::fs::File::create(r.path().join("big.png"))
        .unwrap()
        .set_len(big)
        .unwrap();
    let g = Git::new(r.path());
    let v = image_versions(&g, "big.png", None, &DiffSource::Untracked).unwrap();
    assert_eq!(
        v.new,
        Some(ImageVersion {
            size: big,
            data: None
        })
    );
}
