import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import * as api from '../api';
import type { MenuContext } from '../util/menus';
import type { PullMode } from '../types';
import { pullPlan } from '../util/pull';
import { forcePushWarning } from '../util/push';
import { explainRefName } from '../util/refName';
import { joinNames } from '../util/marks';
import { defaultRemote } from '../util/watchDiff';
import { actionFor, defaultUpstream, isNotMerged, remoteCopyToDelete, type MenuKind } from '../util/actionTable';
import { choose, confirm, prompt } from './modal.svelte';
import { runOp } from './ops';
import { repo } from './repo.svelte';
import { toasts } from './toasts.svelte';

/** MenuContext plus what is needed to carry the action out. */
export interface ActionContext extends MenuContext {
  /** Which menu the id came from; ids such as `checkout` and `delete` are reused across menus. */
  menu: MenuKind;
  /** Repository path. */
  repo: string;
  /** Commit the menu was opened on (branch tip for branch rows). Needed by commit-level actions. */
  oid?: string;
  /** Stash index (stash menu). */
  stashIndex?: number;
}

const message = (e: unknown) => (e as { message?: string } | null)?.message ?? String(e);

/**
 * A Modal `validate` for a branch or tag name: the specific rule a bad name breaks, then git's
 * own check as the authority (error text, or null when valid).
 */
export const validator = (fn: (repo: string, name: string) => Promise<void>, path: string) => async (v: string) => {
  const problem = explainRefName(v);
  if (problem) return problem;
  try {
    await fn(path, v);
    return null;
  } catch (e) {
    const m = message(e);
    return /^Invalid (branch|tag) name/.test(m) ? "Git doesn't accept this name." : m;
  }
};

/** Copy text to the clipboard and confirm with a toast. */
export async function copyText(text: string) {
  try {
    await writeText(text);
    toasts.push({ kind: 'ok', title: 'Copied', detail: text });
  } catch (e) {
    toasts.push({ kind: 'error', title: 'Copy failed', detail: message(e) });
  }
}

/**
 * Apply or pop a stash. A conflicting restore is not a failure: the conflicted files
 * show in Working changes, which is selected, and a pop keeps the stash entry.
 */
export async function restoreStash(path: string, stash: { index: number; oid: string }, pop: boolean) {
  const label = pop ? 'Pop stash' : 'Apply stash';
  const { index, oid } = stash;
  const out = await runOp(label, () => (pop ? api.stashPop(path, index, oid) : api.stashApply(path, index, oid)));
  if (out !== 'conflicts' || repo.path !== path) return;
  repo.selectWip();
  toasts.push({
    kind: 'error',
    title: `${label}: conflicts`,
    detail: `Resolve the conflicted files in Working changes.${pop ? ' The stash was kept.' : ''}`,
  });
}

/**
 * Pull the checked-out branch. Without a usable upstream there is nothing git can pull,
 * so explain why and offer the fix as a button: track the remote's same-named branch, or publish.
 */
export async function pullHead(path: string, mode: PullMode, publish: (branch: string) => unknown) {
  const b = repo.refs.local.find((x) => x.isHead);
  const plan = pullPlan(b, repo.refs.remote.map((r) => r.fullName), repo.refs.remotes);
  const name = b?.name ?? '';
  switch (plan.kind) {
    case 'pull':
      await runOp('Pull', () => api.pull(path, mode));
      return;
    case 'track': {
      const upstream = plan.upstream;
      toasts.push({
        kind: 'info',
        title: `${name} has no upstream`,
        detail: `${upstream} exists on the remote. Track it, then pull.`,
        action: {
          label: 'Track and pull',
          run: () => runOp('Pull', async () => {
            await api.setUpstream(path, name, upstream);
            return api.pull(path, mode);
          }),
        },
      });
      return;
    }
    case 'publish':
      toasts.push({
        kind: 'info',
        title: plan.gone ? `${b?.upstream} was deleted on the remote` : `${name} isn't on ${plan.remote} yet`,
        detail: plan.gone
          ? `There's nothing to pull. Push to publish ${name} again.`
          : `There's nothing to pull. Push to publish it; ${plan.remote}/${name} becomes its upstream.`,
        action: { label: 'Push', run: () => publish(name) },
      });
      return;
    case 'noRemote':
      toasts.push({ kind: 'error', title: 'Nothing to pull', detail: 'This repository has no remote.' });
  }
}

async function deleteBranch(c: ActionContext) {
  const ok = await confirm({
    title: `Delete branch ${c.name}?`,
    body: `The local branch ${c.name} will be deleted.`,
    danger: true,
    confirmLabel: 'Delete',
  });
  if (!ok) return;
  // R4: offer to delete the remote copy too, only when the upstream is <remote>/<same name>
  // and still exists there.
  let remote: string | undefined;
  const upstream = repo.refs.local.find((b) => b.name === c.name)?.upstream;
  const r = remoteCopyToDelete(c.name, upstream, repo.refs.remotes, repo.refs.remote.map((b) => b.fullName));
  if (r && upstream) {
    const also = await confirm({
      title: `Also delete ${upstream} on the remote?`,
      body: `This runs git push ${r} --delete ${c.name}.`,
      danger: true,
      confirmLabel: 'Delete remote branch too',
    });
    if (also) remote = r;
  }
  let notMerged = false;
  await runOp(
    'Delete branch',
    () => api.deleteBranch(c.repo, c.name, false, remote),
    { onError: (e) => (notMerged = isNotMerged(e)) },
  );
  if (!notMerged) return;
  const force = await confirm({
    title: 'Force delete?',
    body: `${c.name} is not fully merged. Its unmerged commits may be lost.`,
    danger: true,
    confirmLabel: 'Force delete',
  });
  if (force) await runOp('Delete branch', () => api.deleteBranch(c.repo, c.name, true, remote));
}

const plural = (n: number, one: string, many = `${one}es`) => `${n} ${n === 1 ? one : many}`;

/**
 * Runs `del` for each name, then throws one error naming those that failed (after a refresh, so
 * the ones that went are gone from the lists). Names git refused as not fully merged go into
 * `notMerged`, for a force delete; when that is all of them, `none` is thrown (the caller claims
 * it: no toast, as the force dialog follows).
 */
async function deleteEach(names: string[], del: (name: string) => Promise<void>, notMerged: string[], none: object) {
  const failed: string[] = [];
  for (const name of names) {
    try {
      await del(name);
    } catch (e) {
      if (isNotMerged(e)) notMerged.push(name);
      else failed.push(`${name}: ${(e as { stderr?: string } | null)?.stderr?.trim() || message(e)}`);
    }
  }
  if (failed.length) {
    await repo.refresh(['Git', 'Worktree']);
    throw { kind: 'failed', message: failed.join('\n') };
  }
  if (notMerged.length === names.length) throw none;
}

/** Deletes several local branches; the checked-out one is left out. Like deleteBranch, with one dialog for them all. */
export async function deleteBranches(path: string, names: string[]) {
  const head = repo.refs.local.find((b) => b.isHead)?.name;
  const todo = names.filter((n) => n !== head);
  if (!todo.length) return;
  const kept = head && names.includes(head) ? ` ${head} is checked out and is kept.` : '';
  const ok = await confirm({
    title: `Delete ${plural(todo.length, 'branch')}?`,
    body: `The local ${todo.length === 1 ? 'branch' : 'branches'} ${joinNames(todo)} will be deleted.${kept}`,
    danger: true,
    confirmLabel: 'Delete',
  });
  if (!ok) return;
  // As for one branch: offer to delete the remote copies, where the upstream is <remote>/<same name>.
  const remoteBranches = repo.refs.remote.map((b) => b.fullName);
  const copies = new Map<string, string>();
  for (const n of todo) {
    const r = remoteCopyToDelete(n, repo.refs.local.find((b) => b.name === n)?.upstream, repo.refs.remotes, remoteBranches);
    if (r) copies.set(n, r);
  }
  if (copies.size) {
    const also = await confirm({
      title: `Also delete ${copies.size === 1 ? 'its copy' : `${copies.size} of them`} on the remote?`,
      body: `This runs git push --delete for ${joinNames([...copies].map(([n, r]) => `${r}/${n}`))}, removing ${copies.size === 1 ? 'it' : 'them'} for everyone who uses the remote.`,
      danger: true,
      confirmLabel: 'Delete remote branches too',
    });
    if (!also) copies.clear();
  }
  const none = {};
  const del = (force: boolean) => (n: string) => api.deleteBranch(path, n, force, copies.get(n));
  const left: string[] = [];
  await runOp('Delete branches', () => deleteEach(todo, del(false), left, none), { onError: (e) => e === none });
  if (!left.length || repo.path !== path) return;
  const force = await confirm({
    title: 'Force delete?',
    body: `${joinNames(left)} ${left.length === 1 ? 'is' : 'are'} not fully merged. Their unmerged commits may be lost.`,
    danger: true,
    confirmLabel: 'Force delete',
  });
  if (force) await runOp('Delete branches', () => deleteEach(left, del(true), [], {}));
}

/** Deletes several branches on their remotes (Remote rows, given as `origin/x`); local branches are kept. */
export async function deleteRemoteBranches(path: string, fullNames: string[]) {
  const rbs = repo.refs.remote.filter((b) => fullNames.includes(b.fullName));
  if (!rbs.length) return;
  const remotes = [...new Set(rbs.map((b) => b.remote))];
  const ok = await confirm({
    title: `Delete ${plural(rbs.length, 'branch')} on ${remotes.length === 1 ? remotes[0] : 'their remotes'}?`,
    body: `This runs git push --delete for ${joinNames(rbs.map((b) => b.fullName))}, removing them for everyone who uses ${joinNames(remotes)}. Local branches that track them are kept and will show as gone.`,
    danger: true,
    confirmLabel: 'Delete on remote',
  });
  if (ok) await runOp('Delete remote branches', () => deleteEach(rbs.map((b) => b.fullName), (n) => api.deleteRemoteBranch(path, n), [], {}));
}

/** Deletes a branch on its remote (the menu on a Remote row); local branches are kept. */
async function deleteRemoteBranch(c: ActionContext) {
  const rb = repo.refs.remote.find((b) => b.fullName === c.name);
  if (!rb) return;
  const tracking = repo.refs.local.filter((b) => b.upstream === rb.fullName).map((b) => b.name);
  const keep = tracking.length
    ? ` Your local ${tracking.length === 1 ? 'branch' : 'branches'} ${tracking.join(', ')} ${tracking.length === 1 ? 'is' : 'are'} kept and will show as gone.`
    : '';
  const ok = await confirm({
    title: `Delete ${rb.name} on ${rb.remote}?`,
    body: `This runs git push ${rb.remote} --delete ${rb.name}, removing it for everyone who uses ${rb.remote}.${keep}`,
    danger: true,
    confirmLabel: 'Delete on remote',
  });
  if (ok) await runOp('Delete remote branch', () => api.deleteRemoteBranch(c.repo, c.name));
}

async function push(c: ActionContext) {
  await runOp('Push', () => api.push(c.repo, c.name, false), {
    onPushRejected: async () => {
      const ok = await confirm({
        title: 'Push rejected',
        body: forcePushWarning(c.name),
        danger: true,
        confirmLabel: 'Force push (with lease)',
      });
      if (ok) await runOp('Force push', () => api.push(c.repo, c.name, true));
    },
  });
}

async function setUpstream(c: ActionContext) {
  const names = repo.refs.remote.map((r) => r.fullName);
  if (names.length === 0) {
    toasts.push({ kind: 'error', title: 'No remote branches', detail: 'Fetch or add a remote first.' });
    return;
  }
  const def = defaultUpstream(c.name, names);
  const ordered = def ? [def, ...names.filter((n) => n !== def)] : names;
  const pick = await choose({
    title: `Set upstream of ${c.name}`,
    options: ordered.map((n) => ({ value: n, label: n === def ? `${n} (default)` : n })),
  });
  if (pick) await runOp('Set upstream', () => api.setUpstream(c.repo, c.name, pick));
}

async function createBranch(c: ActionContext, at: string) {
  const name = await prompt({ title: 'Create branch', label: 'Branch name', validate: validator(api.validateBranchName, c.repo) });
  if (name) await runOp('Create branch', () => api.createBranch(c.repo, name, at, true));
}

async function createTag(c: ActionContext, at: string, annotated: boolean) {
  const name = await prompt({ title: annotated ? 'Create annotated tag' : 'Create tag', label: 'Tag name', validate: validator(api.validateTagName, c.repo) });
  if (!name) return;
  let msg: string | undefined;
  if (annotated) {
    const m = await prompt({
      title: `Message for ${name}`,
      label: 'Tag message',
      validate: async (v) => (v.trim() ? null : 'A message is required'),
    });
    if (m === null) return;
    msg = m;
  }
  await runOp('Create tag', () => api.createTag(c.repo, name, at, msg));
}

async function reset(c: ActionContext, oid: string, mode: 'soft' | 'mixed' | 'hard') {
  if (mode === 'hard') {
    const ok = await confirm({
      title: 'Hard reset?',
      body: `Reset ${c.current ?? 'HEAD'} to ${oid.slice(0, 7)} and discard all uncommitted changes. This cannot be undone.`,
      danger: true,
      confirmLabel: 'Hard reset',
    });
    if (!ok) return;
  }
  await runOp(`Reset (${mode})`, () => api.reset(c.repo, oid, mode));
}

async function rename(c: ActionContext) {
  const name = await prompt({ title: `Rename ${c.name}`, label: 'New branch name', initial: c.name, validate: validator(api.validateBranchName, c.repo) });
  if (name && name !== c.name) await runOp('Rename branch', () => api.renameBranch(c.repo, c.name, name));
}

/**
 * Carry out a context-menu pick: confirmations, prompts, the backend call (via runOp) and clipboard.
 * `id` is the leaf MenuItem id; `ctx.menu` says which menu it came from.
 * Resolves when the action finished or was cancelled; failures are reported as toasts, never thrown.
 */
export async function performMenuAction(id: string, ctx: ActionContext): Promise<void> {
  const key = actionFor(ctx.menu, id);
  if (!key) return;
  const { repo: path, name, oid } = ctx;
  const needOid = (): string | null => {
    if (!oid) toasts.push({ kind: 'error', title: 'No commit selected' });
    return oid ?? null;
  };
  switch (key) {
    case 'pull':
      if (ctx.isHeadBranch) await pullHead(path, 'default', () => push(ctx));
      else await runOp('Pull', () => api.pullBranch(path, name));
      return;
    case 'push': return push(ctx);
    case 'setUpstream': return setUpstream(ctx);
    case 'merge': await runOp('Merge', () => api.merge(path, name)); return;
    case 'rebase': await runOp('Rebase', () => api.rebase(path, name)); return;
    case 'checkoutBranch': await runOp('Checkout', () => api.checkout(path, name, 'local')); return;
    case 'checkoutTag': await runOp('Checkout', () => api.checkout(path, name, 'tag')); return;
    case 'checkoutRemote': await runOp('Checkout', () => api.checkout(path, name, 'remote')); return;
    case 'checkoutCommit': {
      const o = needOid();
      if (o) await runOp('Checkout', () => api.checkoutCommit(path, o));
      return;
    }
    case 'createBranch': {
      const o = needOid();
      if (o) await createBranch(ctx, o);
      return;
    }
    case 'cherryPick': {
      const o = needOid();
      if (o) await runOp('Cherry pick', () => api.cherryPick(path, o));
      return;
    }
    case 'revert': {
      const o = needOid();
      if (o) await runOp('Revert', () => api.revert(path, o));
      return;
    }
    case 'resetSoft':
    case 'resetMixed':
    case 'resetHard': {
      const o = needOid();
      if (o) await reset(ctx, o, key === 'resetSoft' ? 'soft' : key === 'resetMixed' ? 'mixed' : 'hard');
      return;
    }
    case 'rename': return rename(ctx);
    case 'deleteBranch': return deleteBranch(ctx);
    case 'deleteRemoteBranch': return deleteRemoteBranch(ctx);
    case 'deleteTag': {
      const ok = await confirm({ title: `Delete tag ${name}?`, body: `The local tag ${name} will be deleted.`, danger: true, confirmLabel: 'Delete' });
      if (!ok) return;
      let remote: string | undefined;
      const remotes = repo.refs.remotes;
      if (remotes.length === 1) {
        const yes = await confirm({
          title: `Also delete tag ${name} on ${remotes[0]}?`,
          body: `This runs git push ${remotes[0]} --delete ${name}.`,
          danger: true,
          confirmLabel: 'Delete on remote too',
        });
        if (yes) remote = remotes[0];
      } else if (remotes.length > 1) {
        const def = defaultRemote(remotes);
        const ordered = def ? [def, ...remotes.filter((r) => r !== def)] : remotes;
        const pick = await choose({
          title: `Also delete tag ${name} on a remote?`,
          options: [
            { value: '', label: 'Local only' },
            ...ordered.map((r) => ({ value: r, label: `Also on ${r}` })),
          ],
        });
        if (pick) remote = pick;
      }
      await runOp('Delete tag', () => api.deleteTag(path, name, remote));
      return;
    }
    case 'pushTag': await runOp('Push tag', () => api.pushTag(path, name)); return;
    case 'createTag':
    case 'createAnnotatedTag': {
      const o = needOid();
      if (o) await createTag(ctx, o, key === 'createAnnotatedTag');
      return;
    }
    case 'copyBranchName':
    case 'copyTagName': return copyText(name);
    case 'copySha': {
      const o = needOid();
      if (o) await copyText(o);
      return;
    }
    case 'stashApply':
    case 'stashPop':
    case 'stashDrop': {
      const i = ctx.stashIndex;
      const o = ctx.oid;
      if (i === undefined || !o) return;
      if (key === 'stashApply' || key === 'stashPop') await restoreStash(path, { index: i, oid: o }, key === 'stashPop');
      else {
        const ok = await confirm({ title: 'Delete stash?', body: `stash@{${i}} will be permanently dropped.`, danger: true, confirmLabel: 'Delete' });
        if (ok) await runOp('Delete stash', () => api.stashDrop(path, i, o));
      }
      return;
    }
  }
}
