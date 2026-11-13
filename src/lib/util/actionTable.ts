import type { GitError } from '../types';

export type MenuKind = 'branch' | 'remoteBranch' | 'commit' | 'tag' | 'stash';

export type ActionKey =
  | 'pull' | 'push' | 'setUpstream' | 'merge' | 'rebase'
  | 'checkoutBranch' | 'checkoutCommit' | 'checkoutTag' | 'checkoutRemote'
  | 'createBranch' | 'cherryPick' | 'revert' | 'rename' | 'deleteBranch' | 'deleteRemoteBranch' | 'deleteTag'
  | 'resetSoft' | 'resetMixed' | 'resetHard'
  | 'copyBranchName' | 'copySha' | 'copyTagName' | 'createTag' | 'createAnnotatedTag' | 'pushTag'
  | 'stashApply' | 'stashPop' | 'stashDrop';

const common: Record<string, ActionKey> = {
  createBranch: 'createBranch', cherryPick: 'cherryPick', revert: 'revert', copySha: 'copySha',
  createTag: 'createTag', createAnnotatedTag: 'createAnnotatedTag',
  'reset.soft': 'resetSoft', 'reset.mixed': 'resetMixed', 'reset.hard': 'resetHard',
};

/** (menu kind, menu item id) -> action. Ids such as `checkout` and `delete` are reused across menus. */
const TABLE: Record<MenuKind, Record<string, ActionKey>> = {
  branch: {
    ...common,
    pull: 'pull', push: 'push', setUpstream: 'setUpstream', merge: 'merge', rebase: 'rebase',
    checkout: 'checkoutBranch', rename: 'rename', delete: 'deleteBranch', copyBranchName: 'copyBranchName',
  },
  remoteBranch: { checkout: 'checkoutRemote', copyBranchName: 'copyBranchName', delete: 'deleteRemoteBranch' },
  commit: { ...common, checkout: 'checkoutCommit' },
  tag: { checkout: 'checkoutTag', pushTag: 'pushTag', delete: 'deleteTag', copyTagName: 'copyTagName' },
  stash: { stashApply: 'stashApply', stashPop: 'stashPop', stashDrop: 'stashDrop' },
};

export function actionFor(menu: MenuKind, id: string): ActionKey | null {
  return TABLE[menu][id] ?? null;
}

/** Default Set Upstream choice: `origin/<name>` when that remote branch exists. */
export function defaultUpstream(name: string, remoteBranches: string[]): string | null {
  const want = `origin/${name}`;
  return remoteBranches.includes(want) ? want : null;
}

/** Remote part of an upstream like `origin/feat/x` (longest matching remote name). */
export function remoteOfUpstream(upstream: string, remotes: string[]): string | null {
  let best: string | null = null;
  for (const r of remotes) {
    if (upstream.startsWith(`${r}/`) && (best === null || r.length > best.length)) best = r;
  }
  return best;
}

/** `git branch -d` refused because the branch is not merged. */
export function isNotMerged(e: unknown): boolean {
  const err = e as Partial<GitError> | null;
  return err?.kind === 'failed' && /not fully merged/.test(err.stderr ?? '');
}
