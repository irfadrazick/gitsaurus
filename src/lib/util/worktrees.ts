import type { GitError, Worktree } from '../types';

/** Row label: the checked-out branch, else the short commit of a detached HEAD. */
export const worktreeLabel = (w: Worktree) => w.branch ?? w.oid?.slice(0, 7) ?? 'No commits';

/** Branches checked out in a worktree other than this one (git won't switch to them here). */
export function checkedOutElsewhere(worktrees: Worktree[]): Map<string, Worktree> {
  const m = new Map<string, Worktree>();
  for (const w of worktrees) if (w.branch && !w.isCurrent) m.set(w.branch, w);
  return m;
}

/** `git worktree remove` refused because the worktree has uncommitted or untracked changes. */
export function isDirtyWorktreeError(e: unknown): boolean {
  const err = e as Partial<GitError> | null;
  return err?.kind === 'failed' && /contains modified or untracked files/.test(err.stderr ?? '');
}

const trim = (p: string) => (p.length > 1 ? p.replace(/\/+$/, '') : p);
export const samePath = (a: string, b: string) => trim(a) === trim(b);
