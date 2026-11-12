//! Opt-in timing of the graph build against a large repository.
//! `GITSAURUS_PERF_REPO=/path/to/repo cargo test --release --test perf -- --ignored --nocapture`
use std::path::PathBuf;
use std::time::Instant;

use gitsaurus_lib::commands::page;
use gitsaurus_lib::git::Git;
use gitsaurus_lib::git::graph::build_graph;

#[test]
#[ignore = "needs GITSAURUS_PERF_REPO pointing at a large repository"]
fn build_graph_and_first_page_timing() {
    let path = PathBuf::from(
        std::env::var("GITSAURUS_PERF_REPO").expect("set GITSAURUS_PERF_REPO to a repository path"),
    );
    let git = Git::new(&path);
    let t = Instant::now();
    let data = build_graph(&git).expect("build_graph");
    let built = t.elapsed();
    let t = Instant::now();
    let first = page(&data, 0, 200);
    let paged = t.elapsed();
    println!(
        "build_graph: {} commits, {} lanes max, {:?}; first page ({} rows): {:?}",
        data.commits.len(),
        data.max_lanes,
        built,
        first.len(),
        paged
    );
}
