//! Its own test binary: `cancel_network` cancels every running network command in the
//! process, so it must not share a process with other tests' network commands.
mod common;
use common::TestRepo;
use gitsaurus_lib::git::{Git, GitError, cancel_network};
use std::time::{Duration, Instant};

#[test]
fn cancel_stops_running_and_queued_network_commands() {
    let r = TestRepo::new();
    r.git(&["config", "alias.t", "!sleep 30 & wait; true"]);
    let path = r.path().to_path_buf();
    let spawn =
        |p: std::path::PathBuf| std::thread::spawn(move || Git::new(&p).run_network(&["t"]));
    let running = spawn(path.clone());
    std::thread::sleep(Duration::from_millis(200));
    let queued = spawn(path.clone());
    std::thread::sleep(Duration::from_millis(200));
    let start = Instant::now();
    assert_eq!(cancel_network(), 2);
    for h in [running, queued] {
        match h.join().unwrap() {
            Err(GitError::Cancelled { .. }) => {}
            other => panic!("{other:?}"),
        }
    }
    assert!(
        start.elapsed() < Duration::from_secs(5),
        "took {:?}",
        start.elapsed()
    );
    // Nothing is left registered once both have returned.
    assert_eq!(cancel_network(), 0);
}
