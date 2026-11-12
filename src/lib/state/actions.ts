import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import * as api from '../api';
import type { MenuContext } from '../util/menus';
import type { PullMode } from '../types';
import { pullPlan } from '../util/pull';
import { forcePushWarning } from '../util/push';
import { defaultRemote } from '../util/watchDiff';
import { actionFor, defaultUpstream, isNotMerged, remoteOfUpstream, type MenuKind } from '../util/actionTable';
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

/** Map a rejected validate call to the Modal's `validate` contract (error text, or null when valid). */
const validator = (fn: (repo: string, name: string) => Promise<void>, path: string) => async (v: string) => {
  try {
    await fn(path, v);
    return null;
  } catch (e) {
    return message(e);
  }
};

async function copy(text: string) {
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
  // R4: offer to delete the remote copy too, only when the upstream is <remote>/<same name>.
  let remote: string | undefined;
  const upstream = repo.refs.local.find((b) => b.name === c.name)?.upstream;
  if (upstream) {
    const r = remoteOfUpstream(upstream, repo.refs.remotes);
    if (r && upstream === `${r}/${c.name}`) {
      const also = await confirm({
        title: `Also delete ${upstream} on the remote?`,
        body: `This runs git push ${r} --delete ${c.name}.`,
        danger: true,
        confirmLabel: 'Delete remote branch too',
      });
      if (also) remote = r;
    }
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
    case 'copyTagName': return copy(name);
    case 'copySha': {
      const o = needOid();
      if (o) await copy(o);
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
