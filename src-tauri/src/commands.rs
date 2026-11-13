//! Tauri IPC surface. Every command takes `repo` first; blocking git work runs
//! in `spawn_blocking`, and mutating commands hold the per-repo lock.
use std::path::Path;
use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::git as g;
use crate::git::graph::{GraphData, build_graph, search};
use crate::git::ops::{branch, integrate, remote, worktree};
use crate::git::{
    CommitDetails, DiffSource, FileDiff, Git, GitError, GitResult, GraphCommit, GraphSummary,
    OpOutcome, PullMode, RefKind, Refs, RepoInfo, RepoState, ResetMode, Side,
};
use crate::state::AppState;
use crate::{system, watcher};

fn join_err(e: impl std::fmt::Display) -> GitError {
    GitError::Failed {
        message: "internal error".into(),
        stderr: e.to_string(),
    }
}

/// Run blocking read-only work off the async runtime.
async fn read<T, F>(repo: String, f: F) -> GitResult<T>
where
    T: Send + 'static,
    F: FnOnce(Git) -> GitResult<T> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || f(Git::new(Path::new(&repo))))
        .await
        .map_err(join_err)?
}

/// Run blocking mutating work while holding the per-repo lock.
async fn write<T, F>(state: &AppState, repo: String, f: F) -> GitResult<T>
where
    T: Send + 'static,
    F: FnOnce(Git) -> GitResult<T> + Send + 'static,
{
    let lock = state.repo_lock(Path::new(&repo));
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = lock.lock().unwrap_or_else(|e| e.into_inner());
        f(Git::new(Path::new(&repo)))
    })
    .await
    .map_err(join_err)?
}

#[tauri::command]
pub async fn open_repo(repo: String) -> GitResult<RepoInfo> {
    tauri::async_runtime::spawn_blocking(move || g::open_repo(Path::new(&repo)))
        .await
        .map_err(join_err)?
}

// ---- graph ----

fn cached_or_build(state: &AppState, repo: &Path) -> GitResult<Arc<GraphData>> {
    let hit = state
        .graphs
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(repo)
        .cloned();
    if let Some(d) = hit {
        return Ok(d);
    }
    build_and_store(state, repo)
}

fn build_and_store(state: &AppState, repo: &Path) -> GitResult<Arc<GraphData>> {
    let data = Arc::new(build_graph(&Git::new(repo))?);
    state
        .graphs
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(repo.to_path_buf(), data.clone());
    Ok(data)
}

/// Clamp `offset`/`limit` to the available range.
pub fn page(data: &GraphData, offset: u32, limit: u32) -> Vec<GraphCommit> {
    let len = data.commits.len();
    let start = (offset as usize).min(len);
    let end = start.saturating_add(limit as usize).min(len);
    data.commits[start..end].to_vec()
}

#[tauri::command]
pub async fn load_graph(app: AppHandle, repo: String) -> GitResult<GraphSummary> {
    blocking_with_state(app, move |state| {
        let d = build_and_store(state, Path::new(&repo))?;
        Ok(GraphSummary {
            total: d.commits.len() as u32,
            max_lanes: d.max_lanes,
            head_index: d.head_index,
        })
    })
    .await
}

#[tauri::command]
pub async fn graph_page(
    app: AppHandle,
    repo: String,
    offset: u32,
    limit: u32,
) -> GitResult<Vec<GraphCommit>> {
    blocking_with_state(app, move |state| {
        Ok(page(
            &*cached_or_build(state, Path::new(&repo))?,
            offset,
            limit,
        ))
    })
    .await
}

#[tauri::command]
pub async fn find_commit(app: AppHandle, repo: String, oid: String) -> GitResult<Option<u32>> {
    blocking_with_state(app, move |state| {
        Ok(cached_or_build(state, Path::new(&repo))?
            .by_oid
            .get(&oid)
            .copied())
    })
    .await
}

#[tauri::command]
pub async fn search_commits(app: AppHandle, repo: String, query: String) -> GitResult<Vec<u32>> {
    blocking_with_state(app, move |state| {
        Ok(search(&*cached_or_build(state, Path::new(&repo))?, &query))
    })
    .await
}

/// Run blocking work that needs `AppState` (State<'_> cannot cross into spawn_blocking).
async fn blocking_with_state<T, F>(app: AppHandle, f: F) -> GitResult<T>
where
    T: Send + 'static,
    F: FnOnce(&AppState) -> GitResult<T> + Send + 'static,
{
    use tauri::Manager;
    tauri::async_runtime::spawn_blocking(move || f(app.state::<AppState>().inner()))
        .await
        .map_err(join_err)?
}

#[tauri::command]
pub async fn list_refs(repo: String) -> GitResult<Refs> {
    read(repo, move |git| g::list_refs(&git)).await
}

#[tauri::command]
pub async fn repo_state(repo: String) -> GitResult<RepoState> {
    read(repo, move |git| g::repo_state(&git)).await
}

#[tauri::command]
pub async fn commit_details(repo: String, oid: String) -> GitResult<CommitDetails> {
    read(repo, move |git| g::commit_details(&git, &oid)).await
}

#[tauri::command]
pub async fn file_diff(
    repo: String,
    path: String,
    orig_path: Option<String>,
    source: DiffSource,
    context: u32,
) -> GitResult<FileDiff> {
    read(repo, move |git| {
        g::file_diff(&git, &path, orig_path.as_deref(), &source, context)
    })
    .await
}

#[tauri::command]
pub async fn last_commit_message(repo: String) -> GitResult<(String, String)> {
    read(repo, move |git| worktree::last_commit_message(&git)).await
}

#[tauri::command]
pub async fn stage(state: State<'_, AppState>, repo: String, paths: Vec<String>) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        worktree::stage(&git, &paths)
    })
    .await
}

#[tauri::command]
pub async fn stage_all(state: State<'_, AppState>, repo: String) -> GitResult<()> {
    write(state.inner(), repo, move |git| worktree::stage_all(&git)).await
}

#[tauri::command]
pub async fn unstage(
    state: State<'_, AppState>,
    repo: String,
    paths: Vec<String>,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        worktree::unstage(&git, &paths)
    })
    .await
}

#[tauri::command]
pub async fn unstage_all(state: State<'_, AppState>, repo: String) -> GitResult<()> {
    write(state.inner(), repo, move |git| worktree::unstage_all(&git)).await
}

#[tauri::command]
pub async fn discard(
    state: State<'_, AppState>,
    repo: String,
    paths: Vec<String>,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        worktree::discard(&git, &paths)
    })
    .await
}

#[tauri::command]
pub async fn commit(
    state: State<'_, AppState>,
    repo: String,
    summary: String,
    description: String,
    amend: bool,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        worktree::commit(&git, &summary, &description, amend)
    })
    .await
}

#[tauri::command]
pub async fn checkout(
    state: State<'_, AppState>,
    repo: String,
    target: String,
    kind: RefKind,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        branch::checkout(&git, &target, kind)
    })
    .await
}

#[tauri::command]
pub async fn checkout_commit(
    state: State<'_, AppState>,
    repo: String,
    oid: String,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        branch::checkout_commit(&git, &oid)
    })
    .await
}

#[tauri::command]
pub async fn create_branch(
    state: State<'_, AppState>,
    repo: String,
    name: String,
    at: String,
    checkout: bool,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        branch::create_branch(&git, &name, &at, checkout)
    })
    .await
}

#[tauri::command]
pub async fn rename_branch(
    state: State<'_, AppState>,
    repo: String,
    old: String,
    new: String,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        branch::rename_branch(&git, &old, &new)
    })
    .await
}

#[tauri::command]
pub async fn delete_branch(
    state: State<'_, AppState>,
    repo: String,
    name: String,
    force: bool,
    remote: Option<String>,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        branch::delete_branch(&git, &name, force, remote.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn delete_remote_branch(
    state: State<'_, AppState>,
    repo: String,
    name: String,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        branch::delete_remote_branch(&git, &name)
    })
    .await
}

#[tauri::command]
pub async fn validate_branch_name(repo: String, name: String) -> GitResult<()> {
    read(repo, move |git| branch::validate_ref_name(&git, &name)).await
}

#[tauri::command]
pub async fn validate_tag_name(repo: String, name: String) -> GitResult<()> {
    read(repo, move |git| branch::validate_tag_name(&git, &name)).await
}

#[tauri::command]
pub async fn create_tag(
    state: State<'_, AppState>,
    repo: String,
    name: String,
    at: String,
    message: Option<String>,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        branch::create_tag(&git, &name, &at, message.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn delete_tag(
    state: State<'_, AppState>,
    repo: String,
    name: String,
    remote: Option<String>,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        branch::delete_tag(&git, &name, remote.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn push_tag(
    state: State<'_, AppState>,
    repo: String,
    tag: String,
    remote: Option<String>,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        remote::push_tag(&git, &tag, remote.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn reset(
    state: State<'_, AppState>,
    repo: String,
    oid: String,
    mode: ResetMode,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        branch::reset(&git, &oid, mode)
    })
    .await
}

#[tauri::command]
pub async fn stash_push(
    state: State<'_, AppState>,
    repo: String,
    message: Option<String>,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        branch::stash_push(&git, message.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn stash_apply(
    state: State<'_, AppState>,
    repo: String,
    index: u32,
    oid: String,
) -> GitResult<OpOutcome> {
    write(state.inner(), repo, move |git| {
        branch::stash_apply(&git, index, &oid)
    })
    .await
}

#[tauri::command]
pub async fn stash_pop(
    state: State<'_, AppState>,
    repo: String,
    index: u32,
    oid: String,
) -> GitResult<OpOutcome> {
    write(state.inner(), repo, move |git| {
        branch::stash_pop(&git, index, &oid)
    })
    .await
}

#[tauri::command]
pub async fn stash_drop(
    state: State<'_, AppState>,
    repo: String,
    index: u32,
    oid: String,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        branch::stash_drop(&git, index, &oid)
    })
    .await
}

#[tauri::command]
pub async fn merge(
    state: State<'_, AppState>,
    repo: String,
    target: String,
) -> GitResult<OpOutcome> {
    write(state.inner(), repo, move |git| {
        integrate::merge(&git, &target)
    })
    .await
}

#[tauri::command]
pub async fn rebase(
    state: State<'_, AppState>,
    repo: String,
    onto: String,
) -> GitResult<OpOutcome> {
    write(state.inner(), repo, move |git| {
        integrate::rebase(&git, &onto)
    })
    .await
}

#[tauri::command]
pub async fn cherry_pick(
    state: State<'_, AppState>,
    repo: String,
    oid: String,
) -> GitResult<OpOutcome> {
    write(state.inner(), repo, move |git| {
        integrate::cherry_pick(&git, &oid)
    })
    .await
}

#[tauri::command]
pub async fn revert(state: State<'_, AppState>, repo: String, oid: String) -> GitResult<OpOutcome> {
    write(state.inner(), repo, move |git| {
        integrate::revert(&git, &oid)
    })
    .await
}

#[tauri::command]
pub async fn continue_op(state: State<'_, AppState>, repo: String) -> GitResult<OpOutcome> {
    write(state.inner(), repo, move |git| integrate::continue_op(&git)).await
}

#[tauri::command]
pub async fn skip_op(state: State<'_, AppState>, repo: String) -> GitResult<OpOutcome> {
    write(state.inner(), repo, move |git| integrate::skip_op(&git)).await
}

#[tauri::command]
pub async fn abort_op(state: State<'_, AppState>, repo: String) -> GitResult<()> {
    write(state.inner(), repo, move |git| integrate::abort_op(&git)).await
}

#[tauri::command]
pub async fn resolve_conflict(
    state: State<'_, AppState>,
    repo: String,
    path: String,
    side: Side,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        integrate::resolve_conflict(&git, &path, side)
    })
    .await
}

#[tauri::command]
pub async fn mark_resolved(
    state: State<'_, AppState>,
    repo: String,
    path: String,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        integrate::mark_resolved(&git, &path)
    })
    .await
}

/// Runs WITHOUT the repo lock, like a read: a fetch waiting on the network must not block
/// local operations. Network commands on one repo still run one at a time (the runner's
/// network lock); a background fetch skips its turn when one is already running.
#[tauri::command]
pub async fn fetch_all(repo: String, background: bool) -> GitResult<()> {
    read(repo, move |git| {
        if background {
            remote::background_fetch(&git)
        } else {
            remote::fetch_all(&git)
        }
    })
    .await
}

/// Cancels every running or queued network command (the busy toolbar button).
#[tauri::command]
pub fn cancel_network() -> usize {
    crate::git::cancel_network()
}

#[tauri::command]
pub async fn pull(
    state: State<'_, AppState>,
    repo: String,
    mode: PullMode,
) -> GitResult<OpOutcome> {
    write(state.inner(), repo, move |git| remote::pull(&git, mode)).await
}

#[tauri::command]
pub async fn pull_branch(state: State<'_, AppState>, repo: String, local: String) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        remote::pull_branch(&git, &local)
    })
    .await
}

#[tauri::command]
pub async fn push(
    state: State<'_, AppState>,
    repo: String,
    branch: String,
    force: bool,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        remote::push(&git, &branch, force)
    })
    .await
}

#[tauri::command]
pub async fn set_upstream(
    state: State<'_, AppState>,
    repo: String,
    local: String,
    remote_branch: String,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        remote::set_upstream(&git, &local, &remote_branch)
    })
    .await
}

// ---- watcher & system ----

#[tauri::command]
pub async fn watch_repo(app: AppHandle, state: State<'_, AppState>, repo: String) -> GitResult<()> {
    watcher::watch(&app, &state.watchers, Path::new(&repo)).map_err(|e| GitError::Failed {
        message: "could not watch repository".into(),
        stderr: e,
    })
}

#[tauri::command]
pub async fn unwatch_repo(state: State<'_, AppState>, repo: String) -> GitResult<()> {
    watcher::unwatch(&state.watchers, Path::new(&repo));
    Ok(())
}

#[tauri::command]
pub async fn open_in_editor(repo: String, path: String, editor_command: String) -> GitResult<()> {
    system::open_in_editor(Path::new(&repo), &path, &editor_command)
}

/// `path` is absolute here (matches `open -R <path>`).
// ---- worktrees ----

#[tauri::command]
pub async fn remove_worktree(
    state: State<'_, AppState>,
    repo: String,
    path: String,
    force: bool,
) -> GitResult<()> {
    write(state.inner(), repo, move |git| {
        g::remove_worktree(&git, &path, force)
    })
    .await
}

#[tauri::command]
pub async fn prune_worktrees(state: State<'_, AppState>, repo: String) -> GitResult<()> {
    write(state.inner(), repo, move |git| g::prune_worktrees(&git)).await
}

#[tauri::command]
pub async fn reveal_in_finder(repo: String, path: String) -> GitResult<()> {
    let p = Path::new(&path);
    if p.is_absolute() {
        system::reveal_in_finder(p)
    } else {
        system::reveal_in_finder(&Path::new(&repo).join(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn commit(i: u32) -> GraphCommit {
        serde_json::from_value(serde_json::json!({
            "index": i, "oid": format!("{i:040}"), "shortOid": "x", "parents": [],
            "authorName": "a", "authorEmail": "e", "authorTime": 0, "summary": "s",
            "bodyPreview": "", "lane": 0, "color": 0, "into": [], "out": [], "pass": [],
            "width": 1, "refs": []
        }))
        .unwrap()
    }

    fn data(n: u32) -> GraphData {
        GraphData {
            commits: (0..n).map(commit).collect(),
            by_oid: HashMap::new(),
            max_lanes: 1,
            head_index: None,
        }
    }

    #[test]
    fn page_clamps_offset_and_limit() {
        let d = data(5);
        assert_eq!(page(&d, 0, 2).len(), 2);
        assert_eq!(
            page(&d, 3, 10).iter().map(|c| c.index).collect::<Vec<_>>(),
            vec![3, 4]
        );
        assert!(page(&d, 9, 3).is_empty());
        assert_eq!(page(&d, 1, u32::MAX).len(), 4);
    }
}
