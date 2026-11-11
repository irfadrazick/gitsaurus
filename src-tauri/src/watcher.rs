use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{DebounceEventResult, Debouncer, new_debouncer};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ChangeKind {
    Worktree,
    Git,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoChanged {
    pub repo: String,
    pub kinds: Vec<ChangeKind>,
}

pub type WatcherHandle = Debouncer<RecommendedWatcher>;

pub fn classify(repo: &Path, changed: &Path) -> Option<ChangeKind> {
    let rel = changed.strip_prefix(repo).ok()?;
    let mut comps = rel.components();
    if comps.next().is_some_and(|c| c.as_os_str() == ".git") {
        let second = comps.next().map(|c| c.as_os_str().to_owned());
        let ignored_dir = second
            .as_deref()
            .is_some_and(|n| n == "objects" || n == "logs");
        let lock = rel.extension().is_some_and(|e| e == "lock");
        if ignored_dir || lock {
            return None;
        }
        return Some(ChangeKind::Git);
    }
    Some(ChangeKind::Worktree)
}

/// Map key / watch root shared by `watch` and `unwatch`.
pub fn watch_key(repo: &Path) -> PathBuf {
    resolve_root(repo)
}

/// Drop the debouncer for `repo`, stopping its events.
pub fn unwatch(watchers: &Mutex<HashMap<PathBuf, WatcherHandle>>, repo: &Path) {
    let removed = watchers
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(&watch_key(repo));
    drop(removed);
}

/// FSEvents reports canonical paths, so the root must be canonical too.
pub fn resolve_root(repo: &Path) -> PathBuf {
    std::fs::canonicalize(repo).unwrap_or_else(|_| repo.to_path_buf())
}

/// Start watching `repo` (idempotent per path); emits "repo-changed".
pub fn watch(
    app: &AppHandle,
    watchers: &Mutex<HashMap<PathBuf, WatcherHandle>>,
    repo: &Path,
) -> Result<(), String> {
    // Payload carries the caller's original string; the frontend matches on it.
    let original = repo.to_string_lossy().into_owned();
    let repo = watch_key(repo);
    let repo = repo.as_path();
    let mut map = watchers.lock().unwrap_or_else(|e| e.into_inner());
    if map.contains_key(repo) {
        return Ok(());
    }
    let app = app.clone();
    let root = repo.to_path_buf();
    let mut debouncer = new_debouncer(
        Duration::from_millis(300),
        move |res: DebounceEventResult| {
            let Ok(events) = res else { return };
            let mut kinds: Vec<ChangeKind> = Vec::new();
            for e in events {
                if let Some(k) = classify(&root, &e.path)
                    && !kinds.contains(&k)
                {
                    kinds.push(k);
                }
            }
            if !kinds.is_empty() {
                let _ = app.emit(
                    "repo-changed",
                    RepoChanged {
                        repo: original.clone(),
                        kinds,
                    },
                );
            }
        },
    )
    .map_err(|e| e.to_string())?;
    debouncer
        .watcher()
        .watch(repo, RecursiveMode::Recursive)
        .map_err(|e| e.to_string())?;
    map.insert(repo.to_path_buf(), debouncer);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(p: &str) -> Option<ChangeKind> {
        classify(Path::new("/r"), Path::new(p))
    }

    #[test]
    fn ignores_objects_logs_and_locks() {
        assert_eq!(c("/r/.git/objects/ab/cd"), None);
        assert_eq!(c("/r/.git/index.lock"), None);
        assert_eq!(c("/r/.git/logs/HEAD"), None);
    }

    #[test]
    fn git_dir_is_git() {
        assert_eq!(c("/r/.git/refs/heads/main"), Some(ChangeKind::Git));
        assert_eq!(c("/r/.git/HEAD"), Some(ChangeKind::Git));
    }

    #[test]
    fn outside_repo_is_ignored() {
        assert_eq!(c("/elsewhere/src/a.rs"), None);
    }

    #[test]
    fn canonical_root_classifies_git_events_for_symlinked_input() {
        let tmp = tempfile::tempdir().unwrap();
        let given = tmp.path().to_path_buf();
        let root = resolve_root(&given);
        assert_eq!(root, std::fs::canonicalize(&given).unwrap());
        // FSEvents reports canonical paths.
        assert_eq!(
            classify(&root, &root.join(".git/HEAD")),
            Some(ChangeKind::Git)
        );
        assert_eq!(classify(&root, &root.join(".git/objects/ab/cd")), None);
        // On macOS tempdirs live under /var -> /private/var, so the raw root differs.
        if root != given {
            assert_eq!(classify(&given, &root.join(".git/HEAD")), None);
        }
    }

    #[test]
    fn unwatch_with_noncanonical_path_empties_map() {
        let tmp = tempfile::tempdir().unwrap();
        let given = tmp.path().to_path_buf();
        assert_eq!(watch_key(&given), watch_key(&resolve_root(&given)));
        let watchers = Mutex::new(HashMap::new());
        let mut d = new_debouncer(Duration::from_millis(300), |_: DebounceEventResult| {}).unwrap();
        d.watcher().watch(&given, RecursiveMode::Recursive).unwrap();
        watchers.lock().unwrap().insert(watch_key(&given), d);
        // trailing slash / raw form must resolve to the same key
        let mut slashed = given.clone().into_os_string();
        slashed.push("/");
        unwatch(&watchers, Path::new(&slashed));
        assert!(watchers.lock().unwrap().is_empty());
    }

    #[test]
    fn rest_is_worktree() {
        assert_eq!(c("/r/src/a.rs"), Some(ChangeKind::Worktree));
        assert_eq!(c("/r/.gitignore"), Some(ChangeKind::Worktree));
        assert_eq!(c("/r/Cargo.lock"), Some(ChangeKind::Worktree));
    }
}
