import { invoke } from '@tauri-apps/api/core';
import type {
  CommitDetails, DiffSource, FileDiff, GraphCommit, GraphSummary, OpOutcome, PullMode, RefKind,
  Refs, RepoInfo, RepoState, ResetMode, Side,
} from './types';

// One typed function per backend command. Rejections are `GitError` objects.

export const openRepo = (repo: string) => invoke<RepoInfo>('open_repo', { repo });

// graph
export const loadGraph = (repo: string) => invoke<GraphSummary>('load_graph', { repo });
export const graphPage = (repo: string, offset: number, limit: number) =>
  invoke<GraphCommit[]>('graph_page', { repo, offset, limit });
export const findCommit = (repo: string, oid: string) => invoke<number | null>('find_commit', { repo, oid });
export const searchCommits = (repo: string, query: string) => invoke<number[]>('search_commits', { repo, query });

// read
export const listRefs = (repo: string) => invoke<Refs>('list_refs', { repo });
export const repoState = (repo: string) => invoke<RepoState>('repo_state', { repo });
export const commitDetails = (repo: string, oid: string) => invoke<CommitDetails>('commit_details', { repo, oid });
export const fileDiff = (repo: string, path: string, origPath: string | null, source: DiffSource, context: number) =>
  invoke<FileDiff>('file_diff', { repo, path, origPath, source, context });
/** [summary, description] of HEAD's commit message. */
export const lastCommitMessage = (repo: string) => invoke<[string, string]>('last_commit_message', { repo });

// worktree
export const stage = (repo: string, paths: string[]) => invoke<void>('stage', { repo, paths });
export const stageAll = (repo: string) => invoke<void>('stage_all', { repo });
export const unstage = (repo: string, paths: string[]) => invoke<void>('unstage', { repo, paths });
export const unstageAll = (repo: string) => invoke<void>('unstage_all', { repo });
export const discard = (repo: string, paths: string[]) => invoke<void>('discard', { repo, paths });
export const commit = (repo: string, summary: string, description: string, amend: boolean) =>
  invoke<void>('commit', { repo, summary, description, amend });

// branches, tags, stashes
export const checkout = (repo: string, target: string, kind: RefKind) => invoke<void>('checkout', { repo, target, kind });
export const checkoutCommit = (repo: string, oid: string) => invoke<void>('checkout_commit', { repo, oid });
export const createBranch = (repo: string, name: string, at: string, checkout: boolean) =>
  invoke<void>('create_branch', { repo, name, at, checkout });
export const renameBranch = (repo: string, old: string, newName: string) =>
  invoke<void>('rename_branch', { repo, old, new: newName });
export const deleteBranch = (repo: string, name: string, force: boolean, remote?: string) =>
  invoke<void>('delete_branch', { repo, name, force, remote });
export const validateBranchName = (repo: string, name: string) => invoke<void>('validate_branch_name', { repo, name });
export const validateTagName = (repo: string, name: string) => invoke<void>('validate_tag_name', { repo, name });
export const createTag = (repo: string, name: string, at: string, message?: string) =>
  invoke<void>('create_tag', { repo, name, at, message });
export const deleteTag = (repo: string, name: string, remote?: string) => invoke<void>('delete_tag', { repo, name, remote });
export const pushTag = (repo: string, tag: string, remote?: string) => invoke<void>('push_tag', { repo, tag, remote });
export const reset = (repo: string, oid: string, mode: ResetMode) => invoke<void>('reset', { repo, oid, mode });
export const stashPush = (repo: string, message?: string) => invoke<void>('stash_push', { repo, message });
// Stash ops address stash@{index} and pass its oid; the backend refuses when they no longer match.
export const stashApply = (repo: string, index: number, oid: string) =>
  invoke<OpOutcome>('stash_apply', { repo, index, oid });
export const stashPop = (repo: string, index: number, oid: string) =>
  invoke<OpOutcome>('stash_pop', { repo, index, oid });
export const stashDrop = (repo: string, index: number, oid: string) =>
  invoke<void>('stash_drop', { repo, index, oid });

// integrate
export const merge = (repo: string, target: string) => invoke<OpOutcome>('merge', { repo, target });
export const rebase = (repo: string, onto: string) => invoke<OpOutcome>('rebase', { repo, onto });
export const cherryPick = (repo: string, oid: string) => invoke<OpOutcome>('cherry_pick', { repo, oid });
export const revert = (repo: string, oid: string) => invoke<OpOutcome>('revert', { repo, oid });
export const continueOp = (repo: string) => invoke<OpOutcome>('continue_op', { repo });
export const skipOp = (repo: string) => invoke<OpOutcome>('skip_op', { repo });
export const abortOp = (repo: string) => invoke<void>('abort_op', { repo });
export const resolveConflict = (repo: string, path: string, side: Side) =>
  invoke<void>('resolve_conflict', { repo, path, side });
export const markResolved = (repo: string, path: string) => invoke<void>('mark_resolved', { repo, path });

// remote
export const fetchAll = (repo: string) => invoke<void>('fetch_all', { repo });
export const pull = (repo: string, mode: PullMode) => invoke<OpOutcome>('pull', { repo, mode });
export const pullBranch = (repo: string, local: string) => invoke<void>('pull_branch', { repo, local });
export const push = (repo: string, branch: string, force: boolean) => invoke<void>('push', { repo, branch, force });
export const setUpstream = (repo: string, local: string, remoteBranch: string) =>
  invoke<void>('set_upstream', { repo, local, remoteBranch });

// system
export const watchRepo = (repo: string) => invoke<void>('watch_repo', { repo });
export const unwatchRepo = (repo: string) => invoke<void>('unwatch_repo', { repo });
export const openInEditor = (repo: string, path: string, editorCommand: string) =>
  invoke<void>('open_in_editor', { repo, path, editorCommand });
export const revealInFinder = (repo: string, path: string) => invoke<void>('reveal_in_finder', { repo, path });
