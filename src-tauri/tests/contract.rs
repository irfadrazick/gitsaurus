//! Cross-layer contract: serialises a real `repo_state` to the JSON fixture the
//! frontend tests consume (src/lib/util/__fixtures__/repo_state.json).
//!
//! The test fails when the committed fixture no longer matches what the backend
//! produces. Regenerate it with:
//!   GITSAURUS_UPDATE_FIXTURES=1 cargo test --test contract
mod common;
use common::TestRepo;
use gitsaurus_lib::git::{Git, repo_state};
use std::path::PathBuf;

const PLACEHOLDER_OID: &str = "0000000000000000000000000000000000000000";

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/lib/util/__fixtures__/repo_state.json")
}

/// One staged-only, one worktree-only, one MM, one untracked and one conflicted file.
fn build_repo() -> TestRepo {
    let r = TestRepo::new();
    r.write("conflicted.txt", "base\n");
    r.write("worktree-only.txt", "base\n");
    r.write("both.txt", "base\n");
    r.commit_all("base");
    r.git(&["checkout", "-b", "feat"]);
    r.write("conflicted.txt", "feat\n");
    r.commit_all("feat change");
    r.git(&["checkout", "main"]);
    r.write("conflicted.txt", "main\n");
    r.commit_all("main change");
    r.git_allow_fail(&["merge", "feat"]);

    r.write("staged-only.txt", "new\n");
    r.git(&["add", "--", "staged-only.txt"]);
    r.write("worktree-only.txt", "changed\n");
    r.write("both.txt", "staged\n");
    r.git(&["add", "--", "both.txt"]);
    r.write("both.txt", "staged then changed\n");
    r.write("untracked.txt", "u\n");
    r
}

#[test]
fn repo_state_fixture_matches_backend() {
    let r = build_repo();
    let mut st = repo_state(&Git::new(r.path())).unwrap();
    st.files.sort_by(|a, b| a.path.cmp(&b.path));
    if st.head.oid.is_some() {
        st.head.oid = Some(PLACEHOLDER_OID.into());
    }
    let json = serde_json::to_string_pretty(&st).unwrap() + "\n";

    let path = fixture_path();
    if std::env::var_os("GITSAURUS_UPDATE_FIXTURES").is_some() {
        std::fs::write(&path, &json).unwrap();
        return;
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        committed,
        json,
        "{} is stale; regenerate with GITSAURUS_UPDATE_FIXTURES=1 cargo test --test contract",
        path.display()
    );
}
