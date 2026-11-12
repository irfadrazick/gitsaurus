import * as api from '../api';
import type { Worktree } from '../types';
import { isDirtyWorktreeError, samePath, worktreeLabel } from '../util/worktrees';
import { copyText } from './actions';
import { confirm } from './modal.svelte';
import { runOp } from './ops';
import { session } from './session.svelte';
import { toasts } from './toasts.svelte';

const message = (e: unknown) => (e as { message?: string } | null)?.message ?? String(e);
const openPath = (w: Worktree) => session.openRepos.find((p) => samePath(p, w.path));

/** Open a worktree as its own repository in the left column, or switch to it when it is already open. */
export async function openWorktree(w: Worktree) {
  if (w.isCurrent || w.missing) return;
  const already = openPath(w);
  if (already) {
    session.selected = already;
    return;
  }
  try {
    session.addRepo((await api.openRepo(w.path)).path);
  } catch (e) {
    toasts.push({ kind: 'error', title: "Couldn't open worktree", detail: message(e) });
  }
}

async function removeWorktree(repo: string, w: Worktree) {
  const branch = w.branch ? `The branch ${w.branch} is kept.` : `Its commit ${worktreeLabel(w)} stays in history.`;
  const ok = await confirm({
    title: `Remove worktree ${w.name}?`,
    body: `The folder ${w.path} will be deleted. ${branch}`,
    danger: true,
    confirmLabel: 'Remove',
  });
  if (!ok) return;
  // runOp resolves undefined on failure, so each attempt resolves true on success.
  const attempt = (force: boolean) => () => api.removeWorktree(repo, w.path, force).then(() => true);
  let dirty = false;
  let done = await runOp('Remove worktree', attempt(false), {
    onError: (e) => (dirty = isDirtyWorktreeError(e)),
  });
  if (dirty) {
    const force = await confirm({
      title: `${w.name} has changes`,
      body: 'It has uncommitted or untracked changes. Removing it deletes them permanently.',
      danger: true,
      confirmLabel: 'Discard changes and remove',
    });
    if (!force) return;
    done = await runOp('Remove worktree', attempt(true));
  }
  const open = openPath(w);
  if (done && open) session.closeRepo(open);
}

/** Carry out a pick from the worktree context menu. */
export async function performWorktreeAction(id: string, repo: string, w: Worktree) {
  switch (id) {
    case 'open': return openWorktree(w);
    case 'reveal':
      try {
        await api.revealInFinder(repo, w.path);
      } catch (e) {
        toasts.push({ kind: 'error', title: "Couldn't show in Finder", detail: message(e) });
      }
      return;
    case 'copyPath': return copyText(w.path);
    case 'remove': return removeWorktree(repo, w);
    case 'prune': await runOp('Clean up worktrees', () => api.pruneWorktrees(repo)); return;
  }
}
