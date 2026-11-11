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
    let rel = changed.strip_prefix(repo).unwrap_or(changed);
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

/// Start watching `repo` (idempotent per path); emits "repo-changed".
pub fn watch(
    app: &AppHandle,
    watchers: &Mutex<HashMap<PathBuf, WatcherHandle>>,
    repo: &Path,
) -> Result<(), String> {
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
                        repo: root.to_string_lossy().into_owned(),
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
    fn rest_is_worktree() {
        assert_eq!(c("/r/src/a.rs"), Some(ChangeKind::Worktree));
        assert_eq!(c("/r/.gitignore"), Some(ChangeKind::Worktree));
        assert_eq!(c("/r/Cargo.lock"), Some(ChangeKind::Worktree));
    }
}
