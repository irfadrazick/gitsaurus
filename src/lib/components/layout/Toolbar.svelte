<script lang="ts">
  import * as api from '../../api';
  import { restoreStash } from '../../state/actions';
  import { openMenu } from '../../state/contextMenu.svelte';
  import { confirm, prompt } from '../../state/modal.svelte';
  import { busy, runOp } from '../../state/ops';
  import { repo } from '../../state/repo.svelte';
  import { settings } from '../../state/settings.svelte';
  import { ui } from '../../state/ui.svelte';
  import type { GitError, PullMode } from '../../types';
  import { fetchIntervalMs, shouldFetch } from '../../util/backgroundFetch';
  import type { MenuItem } from '../../util/menus';
  import { forcePushBody } from '../../util/push';

  const path = $derived(repo.path);
  const name = $derived(path ? (path.split('/').filter(Boolean).pop() ?? path) : '');
  const branch = $derived(repo.state.head.branch);
  const oid = $derived(repo.state.head.oid);
  const detached = $derived(branch === null);
  const hasStash = $derived(repo.refs.stashes.length > 0);

  const PULL_ITEMS: MenuItem[] = [
    { id: 'fetch', label: 'Fetch all' },
    { id: 'default', label: 'Pull (fast-forward if possible)', separatorBefore: true },
    { id: 'ffOnly', label: 'Pull (fast-forward only)' },
    { id: 'rebase', label: 'Pull (rebase)' },
  ];

  const msg = (e: unknown) => (e as Partial<GitError>)?.message ?? String(e);

  // Background fetch: silent, skipped while any foreground op is running.
  $effect(() => {
    const p = path;
    const min = settings.fetchIntervalMin;
    const ms = fetchIntervalMs(min);
    if (!p || ms === null) return;
    const id = setInterval(async () => {
      if (!shouldFetch(busy.size > 0, min) || repo.path !== p) return;
      try {
        await api.fetchAll(p);
        if (repo.path === p) await repo.refresh(['Git']);
      } catch (e) {
        console.warn('background fetch failed', e);
      }
    }, ms);
    return () => clearInterval(id);
  });

  function pullMode(mode: PullMode) {
    if (path) runOp('Pull', () => api.pull(path, mode));
  }

  function pullMenu(e: MouseEvent) {
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    openMenu(r.left, r.bottom + 2, PULL_ITEMS, (id) => {
      if (!path) return;
      if (id === 'fetch') runOp('Fetch', () => api.fetchAll(path));
      else pullMode(id as PullMode);
    });
  }

  async function push() {
    const p = path;
    const b = branch;
    if (!p || !b) return;
    await runOp('Push', () => api.push(p, b, false), {
      // onPushRejected carries no payload, so claim the error here to get at its stderr.
      onError: (e) => {
        const err = e as Partial<GitError> & { stderr?: string };
        if (err?.kind !== 'pushRejected') return false;
        void forcePush(p, b, forcePushBody(b, err.stderr || err.message || ''));
        return true;
      },
    });
  }

  async function forcePush(p: string, b: string, body: string) {
    const ok = await confirm({ danger: true, title: 'Push rejected', body, confirmLabel: 'Force push (with lease)' });
    if (ok) await runOp('Push', () => api.push(p, b, true));
  }

  async function newBranch() {
    const p = path;
    if (!p) return;
    const n = await prompt({
      title: 'Create branch',
      label: 'Branch name',
      validate: async (v) => {
        try {
          await api.validateBranchName(p, v);
          return null;
        } catch (e) {
          return msg(e);
        }
      },
    });
    if (n) await runOp('Create branch', () => api.createBranch(p, n, 'HEAD', true));
  }

  async function stash() {
    const p = path;
    if (!p) return;
    const m = await prompt({ title: 'Stash changes', label: 'Message (optional)' });
    if (m !== null) await runOp('Stash', () => api.stashPush(p, m.trim() || undefined));
  }

  function search() {
    repo.showGraph();
    ui.searchOpen = true;
  }
</script>

{#snippet spinner()}<span class="spin" aria-hidden="true"></span>{/snippet}

{#if path}
  <header class="bar">
    <div class="where">
      <span class="repo">{name}</span>
      {#if branch}
        <span class="sep">/</span><span class="branch">{branch}</span>
      {:else}
        <span class="sep">/</span><span class="branch mono">{oid?.slice(0, 7) ?? ''}</span><span class="detached">detached</span>
      {/if}
    </div>
    <div class="actions">
      <div class="split">
        <button class="tb" onclick={() => pullMode('default')} title="Pull">
          {#if busy.get('Pull')}{@render spinner()}{:else}
            <svg viewBox="0 0 16 16"><path d="M8 2v9M4.5 7.5 8 11l3.5-3.5M3 14h10" /></svg>
          {/if}
          <span>Pull</span>
        </button>
        <button class="caret" onclick={pullMenu} aria-label="Pull options" title="Pull options">
          {#if busy.get('Fetch')}{@render spinner()}{:else}▾{/if}
        </button>
      </div>
      <button class="tb" onclick={push} disabled={detached} title="Push">
        {#if busy.get('Push')}{@render spinner()}{:else}
          <svg viewBox="0 0 16 16"><path d="M8 11V2M4.5 5.5 8 2l3.5 3.5M3 14h10" /></svg>
        {/if}
        <span>Push</span>
      </button>
      <button class="tb" onclick={newBranch} title="Create branch">
        {#if busy.get('Create branch')}{@render spinner()}{:else}
          <svg viewBox="0 0 16 16"><circle cx="4.5" cy="3.5" r="1.5" /><circle cx="4.5" cy="12.5" r="1.5" /><circle cx="11.5" cy="5.5" r="1.5" /><path d="M4.5 5v6M11.5 7c0 3-7 1.5-7 3.5" /></svg>
        {/if}
        <span>Branch</span>
      </button>
      <button class="tb" onclick={stash} title="Stash changes">
        {#if busy.get('Stash')}{@render spinner()}{:else}
          <svg viewBox="0 0 16 16"><path d="M2.5 9 4 3h8l1.5 6M2.5 9v4h11V9M2.5 9h3.5l.5 1.5h3L10 9h3.5" /></svg>
        {/if}
        <span>Stash</span>
      </button>
      <button class="tb" onclick={() => restoreStash(path, 0, true)} disabled={!hasStash} title="Pop latest stash">
        {#if busy.get('Pop stash')}{@render spinner()}{:else}
          <svg viewBox="0 0 16 16"><path d="M2.5 9v4h11V9M8 10V2M4.5 5.5 8 2l3.5 3.5" /></svg>
        {/if}
        <span>Pop</span>
      </button>
      <button class="tb" onclick={() => api.openTerminal(path).catch((e) => console.warn(e))} title="Open Terminal here">
        <svg viewBox="0 0 16 16"><rect x="1.5" y="2.5" width="13" height="11" rx="1.5" /><path d="m4.5 6 2.5 2-2.5 2M8.5 10.5h3" /></svg>
        <span>Terminal</span>
      </button>
      <button class="tb" onclick={search} title="Search commits (⌘F)">
        <svg viewBox="0 0 16 16"><circle cx="7" cy="7" r="4.5" /><path d="m10.5 10.5 3.5 3.5" /></svg>
        <span>Search</span>
      </button>
    </div>
  </header>
{/if}

<style>
  .bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--line);
    background: var(--panel);
    min-height: 52px;
  }
  .where {
    display: flex;
    align-items: baseline;
    gap: 6px;
    min-width: 0;
    overflow: hidden;
    white-space: nowrap;
  }
  .repo {
    font-weight: 600;
  }
  .sep,
  .detached {
    color: var(--muted);
  }
  .detached {
    font-size: 11px;
  }
  .branch {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .mono {
    font-family: var(--mono);
  }
  .actions {
    display: flex;
    gap: 2px;
    flex-shrink: 0;
  }
  .split {
    display: flex;
    align-items: stretch;
  }
  .tb,
  .caret {
    background: transparent;
    border: 0;
    color: var(--text);
    cursor: default;
    border-radius: 6px;
  }
  .tb {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
    padding: 4px 9px;
    min-width: 52px;
  }
  .tb span {
    font-size: 11px;
    color: var(--muted);
  }
  .tb svg,
  .spin {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.4;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .caret {
    padding: 0 3px;
    font-size: 11px;
    color: var(--muted);
    margin-left: -6px;
    align-self: flex-start;
    margin-top: 4px;
  }
  .tb:hover:not(:disabled),
  .caret:hover {
    background: rgb(255 255 255 / 0.06);
  }
  .tb:disabled {
    opacity: 0.4;
  }
  .spin {
    border: 2px solid var(--line);
    border-top-color: var(--text);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    box-sizing: border-box;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
