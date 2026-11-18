<script lang="ts" module>
  /** Repo whose background fetch is running (module-level so it survives a remount); ticks never overlap. */
  const bg = $state<{ fetching: string | null }>({ fetching: null });
</script>

<script lang="ts">
  import * as api from '../../api';
  import { pullHead, restoreStash, validator } from '../../state/actions';
  import { onCommand } from '../../state/commands';
  import { openMenu } from '../../state/contextMenu.svelte';
  import { confirm, prompt } from '../../state/modal.svelte';
  import { busy, runOp } from '../../state/ops';
  import { repo } from '../../state/repo.svelte';
  import { settings } from '../../state/settings.svelte';
  import { ui } from '../../state/ui.svelte';
  import Icon from '../Icon.svelte';
  import type { GitError, PullMode } from '../../types';
  import { fetchIntervalMs, shouldFetch } from '../../util/backgroundFetch';
  import type { MenuItem } from '../../util/menus';
  import { forcePushBody } from '../../util/push';

  const path = $derived(repo.path);
  const name = $derived(path ? (path.split('/').filter(Boolean).pop() ?? path) : '');
  const branch = $derived(repo.state.head.branch);
  const oid = $derived(repo.state.head.oid);
  const detached = $derived(branch === null);
  const latestStash = $derived(repo.refs.stashes.find((s) => s.index === 0) ?? null);

  const PULL_ITEMS: MenuItem[] = [
    { id: 'fetch', label: 'Fetch all', hint: '⌘⇧F' },
    { id: 'default', label: 'Pull (fast-forward if possible)', separatorBefore: true, hint: 'Default · ⌘⇧P' },
    { id: 'ffOnly', label: 'Pull (fast-forward only)' },
    { id: 'rebase', label: 'Pull (rebase)' },
  ];


  // While a network command runs, its toolbar button becomes Cancel (they have no time limit).
  const pulling = $derived(busy.has('Pull') || busy.has('Fetch'));
  const pushing = $derived(busy.has('Push') || busy.has('Force push'));
  function cancelNetwork() {
    api.cancelNetwork().catch((e) => console.warn('cancel failed', e));
  }

  // Background fetch: silent, skipped while any foreground op is running.
  $effect(() => {
    const p = path;
    const min = settings.fetchIntervalMin;
    const ms = fetchIntervalMs(min);
    if (!p || ms === null) return;
    const id = setInterval(async () => {
      if (!shouldFetch(busy.size > 0, min, bg.fetching !== null) || repo.path !== p) return;
      bg.fetching = p;
      try {
        await api.fetchAll(p, true);
        if (repo.path === p) await repo.refresh(['Git']);
      } catch (e) {
        console.warn('background fetch failed', e);
      } finally {
        bg.fetching = null;
      }
    }, ms);
    return () => clearInterval(id);
  });

  // Keyboard shortcuts (App) run the same handlers as the buttons, with the same busy guards.
  $effect(() => {
    const offs = [
      onCommand('pull', () => (pulling ? undefined : pullMode('default'))),
      onCommand('push', () => (pushing ? undefined : push())),
      onCommand('fetch', () => {
        const p = path;
        if (p && !pulling) runOp('Fetch', () => api.fetchAll(p));
      }),
      onCommand('newBranch', () => void newBranch()),
    ];
    return () => offs.forEach((off) => off());
  });

  function pullMode(mode: PullMode) {
    // Busy guard: a double-click (or the menu) must not queue a second pull.
    const p = path;
    if (p && !busy.has('Pull')) void pullHead(p, mode, (b) => push(p, b));
  }

  function pullMenu(e: MouseEvent) {
    // Anchor under the whole Pull button, not just the caret.
    const r = ((e.currentTarget as HTMLElement).parentElement ?? (e.currentTarget as HTMLElement)).getBoundingClientRect();
    openMenu(r.left, r.bottom + 2, PULL_ITEMS, (id) => {
      if (!path) return;
      if (id === 'fetch') runOp('Fetch', () => api.fetchAll(path));
      else pullMode(id as PullMode);
    });
  }

  // Takes the repo and branch explicitly when called later (the pull toast's Push button).
  async function push(p = path, b = branch) {
    if (!p || !b || busy.has('Push')) return;
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
    const n = await prompt({ title: 'Create branch', label: 'Branch name', validate: validator(api.validateBranchName, p) });
    if (n) await runOp('Create branch', () => api.createBranch(p, n, 'HEAD', true));
  }

  async function stash() {
    const p = path;
    if (!p) return;
    const m = await prompt({ title: 'Stash changes', label: 'Message (optional)' });
    if (m !== null) await runOp('Stash', () => api.stashPush(p, m.trim() || undefined));
  }

  // The Search button toggles: pressed while the bar is visible over the graph, click again to close.
  const searchShown = $derived(ui.searchOpen && repo.centre.kind === 'graph');
  function search() {
    if (searchShown) {
      ui.searchOpen = false;
      return;
    }
    repo.showGraph();
    ui.searchOpen = true;
    ui.searchFocus++;
  }
</script>

{#snippet spinner()}<span class="spin" aria-hidden="true"></span>{/snippet}

{#if path}
  <!-- The bar is the window's title bar: its empty parts and the name drag the window. -->
  <header class="bar" data-tauri-drag-region>
    <div class="where" data-tauri-drag-region>
      <span class="repo" data-tauri-drag-region>{name}</span>
      {#if branch}
        <span class="sep" data-tauri-drag-region>/</span><span class="branch" data-tauri-drag-region>{branch}</span>
      {:else if oid}
        <!-- Detached HEAD. While a repository opens there is no HEAD yet, so only its name shows. -->
        <span class="sep" data-tauri-drag-region>/</span><span class="branch mono" data-tauri-drag-region>{oid.slice(0, 7)}</span><span class="detached" data-tauri-drag-region>detached</span>
      {/if}
      {#if bg.fetching === path}<span class="fetching" role="status">Fetching…</span>{/if}
    </div>
    <div class="actions">
      <div class="split">
        <button class="tb" onclick={() => (pulling ? cancelNetwork() : pullMode('default'))} title={pulling ? 'Cancel' : 'Pull (⌘⇧P)'}>
          {#if pulling}{@render spinner()}{:else}
            <svg viewBox="0 0 16 16"><path d="M8 2v9M4.5 7.5 8 11l3.5-3.5M3 14h10" /></svg>
          {/if}
          <span>{pulling ? 'Cancel' : 'Pull'}</span>
        </button>
        <button class="caret" onclick={pullMenu} disabled={pulling} aria-label="Pull options" title="Pull options"><Icon name="chevronDown" size={12} /></button>
      </div>
      <button class="tb" onclick={() => (pushing ? cancelNetwork() : push())} disabled={detached && !pushing} title={pushing ? 'Cancel' : 'Push (⌘P)'}>
        {#if pushing}{@render spinner()}{:else}
          <svg viewBox="0 0 16 16"><path d="M8 11V2M4.5 5.5 8 2l3.5 3.5M3 14h10" /></svg>
        {/if}
        <span>{pushing ? 'Cancel' : 'Push'}</span>
      </button>
      <button class="tb" onclick={newBranch} title="Create branch (⌘⇧N)">
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
      <button class="tb" onclick={() => latestStash && restoreStash(path, latestStash, true)} disabled={!latestStash} title="Pop latest stash">
        {#if busy.get('Pop stash')}{@render spinner()}{:else}
          <svg viewBox="0 0 16 16"><path d="M2.5 9v4h11V9M8 10V2M4.5 5.5 8 2l3.5 3.5" /></svg>
        {/if}
        <span>Pop</span>
      </button>
      <button class="tb" class:on={searchShown} aria-pressed={searchShown} onclick={search} title={searchShown ? 'Close search (Esc)' : 'Search commits (⌘F)'}>
        <svg viewBox="0 0 16 16"><circle cx="7" cy="7" r="4.5" /><path d="m10.5 10.5 3.5 3.5" /></svg>
        <span>Search</span>
      </button>
    </div>
  </header>
{/if}

<style>
  /* Repository and branch on the left; the buttons centred in the window (the empty third column
     balances the first, so they stay centred whatever the name's length). */
  .bar {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
    align-items: center;
    gap: 12px;
    /* The left inset leaves room for the window's close, minimise and zoom buttons. */
    padding: 6px 10px 6px var(--traffic-lights-w);
    border-bottom: 1px solid var(--line);
    background: var(--panel);
    min-height: var(--titlebar-h);
  }
  .where {
    display: flex;
    align-items: baseline;
    gap: 6px;
    font-size: var(--fs-title);
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
  .detached,
  .fetching {
    font-size: var(--fs-xs);
  }
  .fetching {
    color: var(--muted);
    margin-left: 4px;
  }
  .branch {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .actions {
    display: flex;
    gap: 2px;
    flex-shrink: 0;
  }
  .split {
    display: flex;
    align-items: stretch;
    border-radius: var(--radius);
  }
  .split:hover:not(:has(:disabled)) {
    background: var(--hover);
  }
  .split .tb {
    border-radius: var(--radius) 0 0 var(--radius);
    padding-right: 4px;
  }
  .tb,
  .caret {
    background: transparent;
    border: 0;
    color: var(--text);
    cursor: default;
    border-radius: var(--radius);
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
    font-size: var(--fs-xs);
    color: var(--muted);
  }
  .tb:hover:not(:disabled) span {
    color: var(--text);
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
    display: grid;
    place-items: center;
    width: 18px;
    padding: 0;
    border-radius: 0 var(--radius) var(--radius) 0;
    color: var(--muted);
  }
  .tb:hover:not(:disabled),
  .caret:hover:not(:disabled) {
    background: var(--hover);
  }
  .tb:active:not(:disabled),
  .caret:active:not(:disabled) {
    background: var(--active);
  }
  .split .tb:hover:not(:disabled),
  .split .caret:hover:not(:disabled) {
    background: var(--hover);
  }
  .tb.on {
    background: var(--active);
  }
  .tb.on span {
    color: var(--text);
  }
  .tb:disabled,
  .caret:disabled {
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
