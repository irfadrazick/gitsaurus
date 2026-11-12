use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{DebounceEventResult, Debouncer, new_debouncer};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::git::Git;

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
        return git_dir_kind(comps.as_path());
    }
    Some(ChangeKind::Worktree)
}

/// Kind of a change at `rel` inside a git dir: objects/, logs/ and *.lock are noise.
/// Under worktrees/ (other worktrees' private dirs) only a worktree being added or
/// removed, or switching its HEAD, matters: that is what the Worktrees list shows.
fn git_dir_kind(rel: &Path) -> Option<ChangeKind> {
    let first = rel.components().next().map(|c| c.as_os_str().to_owned());
    if first.as_deref().is_some_and(|n| n == "worktrees") {
        let parts: Vec<_> = rel.components().map(|c| c.as_os_str()).collect();
        let listed = match parts.len() {
            2 => true,
            3 => parts[2] == "HEAD",
            _ => false,
        };
        return listed.then_some(ChangeKind::Git);
    }
    let ignored_dir = first
        .as_deref()
        .is_some_and(|n| n == "objects" || n == "logs");
    let lock = rel.extension().is_some_and(|e| e == "lock");
    if ignored_dir || lock {
        None
    } else {
        Some(ChangeKind::Git)
    }
}

/// What to watch for one repo: the (canonical) worktree root, plus git dirs that lie
/// outside it. For a linked worktree those are its private git dir
/// (`<main>/.git/worktrees/<name>`) and the common dir (`<main>/.git`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchRoots {
    pub root: PathBuf,
    pub git_dirs: Vec<PathBuf>,
}

impl WatchRoots {
    pub fn classify(&self, changed: &Path) -> Option<ChangeKind> {
        if changed.starts_with(&self.root) {
            return classify(&self.root, changed);
        }
        // The most specific dir wins: a linked worktree's own git dir lies inside the common dir.
        let rel = self
            .git_dirs
            .iter()
            .filter_map(|d| changed.strip_prefix(d).ok())
            .min_by_key(|rel| rel.components().count())?;
        // Under the common dir, worktrees/<name> is another worktree's private git dir
        // (ours, if this is a linked worktree, matched above as the more specific dir).
        git_dir_kind(rel)
    }

    /// Directories to register with the watcher (none nested in another).
    fn watch_paths(&self) -> Vec<&Path> {
        let all: Vec<&Path> = std::iter::once(self.root.as_path())
            .chain(self.git_dirs.iter().map(PathBuf::as_path))
            .collect();
        all.iter()
            .copied()
            .filter(|p| !all.iter().any(|q| q != p && p.starts_with(q)))
            .collect()
    }
}

/// Resolve the watch root and any git dirs outside it (`rev-parse --git-dir --git-common-dir`).
pub fn watch_roots(repo: &Path) -> WatchRoots {
    let root = resolve_root(repo);
    let mut git_dirs = Vec::new();
    if let Ok(out) = Git::new(&root).run_read(&["rev-parse", "--git-dir", "--git-common-dir"]) {
        for line in out.lines().filter(|l| !l.is_empty()) {
            let p = Path::new(line);
            let p = if p.is_absolute() {
                p.to_path_buf()
            } else {
                root.join(p)
            };
            let p = resolve_root(&p);
            if !p.starts_with(&root) && !git_dirs.contains(&p) {
                git_dirs.push(p);
            }
        }
    }
    WatchRoots { root, git_dirs }
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
    let roots = watch_roots(repo);
    let paths: Vec<PathBuf> = roots
        .watch_paths()
        .into_iter()
        .map(Path::to_path_buf)
        .collect();
    let mut debouncer = new_debouncer(
        Duration::from_millis(300),
        move |res: DebounceEventResult| {
            let Ok(events) = res else { return };
            let mut kinds: Vec<ChangeKind> = Vec::new();
            for e in events {
                if let Some(k) = roots.classify(&e.path)
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
    for p in &paths {
        debouncer
            .watcher()
            .watch(p, RecursiveMode::Recursive)
            .map_err(|e| e.to_string())?;
    }
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
    fn main_repo_sees_worktrees_come_and_go_but_not_their_churn() {
        assert_eq!(c("/r/.git/worktrees/wt"), Some(ChangeKind::Git));
        assert_eq!(c("/r/.git/worktrees/wt/HEAD"), Some(ChangeKind::Git));
        assert_eq!(c("/r/.git/worktrees/wt/index"), None);
        assert_eq!(c("/r/.git/worktrees/wt/logs/HEAD"), None);
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

    fn roots() -> WatchRoots {
        WatchRoots {
            root: PathBuf::from("/wt"),
            git_dirs: vec![
                PathBuf::from("/main/.git/worktrees/wt"),
                PathBuf::from("/main/.git"),
            ],
        }
    }

    #[test]
    fn linked_worktree_git_dirs_classify_as_git() {
        let r = roots();
        let k = |p: &str| r.classify(Path::new(p));
        assert_eq!(k("/wt/src/a.rs"), Some(ChangeKind::Worktree));
        assert_eq!(k("/wt/.git"), Some(ChangeKind::Git));
        assert_eq!(k("/main/.git/worktrees/wt/HEAD"), Some(ChangeKind::Git));
        assert_eq!(k("/main/.git/worktrees/wt/index"), Some(ChangeKind::Git));
        assert_eq!(k("/main/.git/refs/heads/x"), Some(ChangeKind::Git));
        assert_eq!(k("/main/.git/packed-refs"), Some(ChangeKind::Git));
        // Still ignored: objects/, logs/ and locks, in either dir.
        assert_eq!(k("/main/.git/objects/ab/cd"), None);
        assert_eq!(k("/main/.git/logs/HEAD"), None);
        assert_eq!(k("/main/.git/worktrees/wt/logs/HEAD"), None);
        assert_eq!(k("/main/.git/worktrees/wt/index.lock"), None);
        assert_eq!(k("/main/.git/refs/heads/x.lock"), None);
        // Other worktrees' private dirs are not ours, except what the Worktrees list
        // shows: one being added or removed, or switching branch.
        assert_eq!(k("/main/.git/worktrees/other/index"), None);
        assert_eq!(k("/main/.git/worktrees/other/HEAD"), Some(ChangeKind::Git));
        assert_eq!(k("/main/.git/worktrees/new"), Some(ChangeKind::Git));
        assert_eq!(k("/main/.git/worktrees/new/HEAD.lock"), None);
        // The main worktree's files are not ours either.
        assert_eq!(k("/main/src/a.rs"), None);
        // The private git dir is inside the common dir, so only the latter is registered.
        assert_eq!(
            r.watch_paths(),
            vec![Path::new("/wt"), Path::new("/main/.git")]
        );
    }

    fn git(dir: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .current_dir(dir)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .args(["-c", "user.name=T", "-c", "user.email=t@example.com"])
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    #[test]
    fn watch_roots_finds_a_linked_worktrees_git_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let main = tmp.path().join("main");
        std::fs::create_dir(&main).unwrap();
        git(&main, &["init", "-q", "-b", "main"]);
        std::fs::write(main.join("f.txt"), "x\n").unwrap();
        git(&main, &["add", "f.txt"]);
        git(&main, &["commit", "-q", "-m", "c"]);
        git(&main, &["worktree", "add", "-q", "-b", "x", "../wt"]);
        let canon = |p: &Path| std::fs::canonicalize(p).unwrap();

        let r = watch_roots(&tmp.path().join("wt"));
        assert_eq!(r.root, canon(&tmp.path().join("wt")));
        assert_eq!(
            r.git_dirs,
            vec![
                canon(&main.join(".git/worktrees/wt")),
                canon(&main.join(".git"))
            ]
        );
        // The main worktree's git dir lies inside its root: nothing extra to watch.
        let m = watch_roots(&main);
        assert_eq!(m.root, canon(&main));
        assert!(m.git_dirs.is_empty(), "{:?}", m.git_dirs);
    }

    #[test]
    fn rest_is_worktree() {
        assert_eq!(c("/r/src/a.rs"), Some(ChangeKind::Worktree));
        assert_eq!(c("/r/.gitignore"), Some(ChangeKind::Worktree));
        assert_eq!(c("/r/Cargo.lock"), Some(ChangeKind::Worktree));
    }
}
