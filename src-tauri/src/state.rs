use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::git::graph::GraphData;
use crate::watcher::WatcherHandle;

#[derive(Default)]
pub struct AppState {
    pub graphs: Mutex<HashMap<PathBuf, Arc<GraphData>>>,
    pub locks: Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>,
    pub watchers: Mutex<HashMap<PathBuf, WatcherHandle>>,
}

impl AppState {
    /// Per-repo mutex serialising mutating operations.
    pub fn repo_lock(&self, repo: &Path) -> Arc<Mutex<()>> {
        let mut locks = self.locks.lock().unwrap_or_else(|e| e.into_inner());
        locks.entry(repo.to_path_buf()).or_default().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_lock_is_shared_per_path() {
        let s = AppState::default();
        let a = s.repo_lock(Path::new("/r1"));
        assert!(Arc::ptr_eq(&a, &s.repo_lock(Path::new("/r1"))));
        assert!(!Arc::ptr_eq(&a, &s.repo_lock(Path::new("/r2"))));
    }
}
