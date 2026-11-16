<script lang="ts">
  import { runCommand } from './lib/state/commands';
  import { modalState } from './lib/state/modal.svelte';
  import { runOp } from './lib/state/ops';
  import { listen } from '@tauri-apps/api/event';
  import { onMount, untrack } from 'svelte';
  import * as api from './lib/api';
  import CommitDetails from './lib/components/details/CommitDetails.svelte';
  import DiffView from './lib/components/diff/DiffView.svelte';
  import GraphView from './lib/components/graph/GraphView.svelte';
  import Columns from './lib/components/layout/Columns.svelte';
  import ContextMenu from './lib/components/layout/ContextMenu.svelte';
  import Settings from './lib/components/layout/Settings.svelte';
  import Toolbar from './lib/components/layout/Toolbar.svelte';
  import Modal from './lib/components/layout/Modal.svelte';
  import Toasts from './lib/components/layout/Toasts.svelte';
  import RefsColumn from './lib/components/refs/RefsColumn.svelte';
  import RepoColumn from './lib/components/repos/RepoColumn.svelte';
  import ConflictBanner from './lib/components/wip/ConflictBanner.svelte';
  import ConflictPanel from './lib/components/wip/ConflictPanel.svelte';
  import WorkingChanges from './lib/components/wip/WorkingChanges.svelte';
  import { repo } from './lib/state/repo.svelte';
  import { health } from './lib/state/repoHealth.svelte';
  import { openRepoDialog } from './lib/state/repos';
  import { session } from './lib/state/session.svelte';
  import { settings } from './lib/state/settings.svelte';
  import { ui } from './lib/state/ui.svelte';
  import { updater } from './lib/state/updater.svelte';
  import type { ChangeKind, RepoChanged } from './lib/types';
  import { isGitAction, matchShortcut } from './lib/util/shortcuts';
  import { watchDiff } from './lib/util/watchDiff';

  onMount(() => {
    session.init();
    settings.init();
    updater.init();
    let unlisten: (() => void) | undefined;
    listen<RepoChanged>('repo-changed', (ev) => {
      if (ev.payload.repo !== repo.path) {
        health.check(ev.payload.repo);
        return;
      }
      const kinds: ChangeKind[] = ev.payload.kinds.map((k) => (k === 'git' ? 'Git' : 'Worktree'));
      repo.refresh(kinds);
    })
      .then((u) => (unlisten = u))
      .catch(() => {});
    // Catch changes the watcher missed (e.g. made while the app was in the background).
    const onFocus = () => {
      health.checkAll([...session.openRepos]);
      if (repo.path) repo.refresh(['Git', 'Worktree']);
    };
    window.addEventListener('focus', onFocus);
    return () => {
      unlisten?.();
      window.removeEventListener('focus', onFocus);
    };
  });

  // Background update checks, while the setting is on.
  $effect(() => {
    if (!settings.loaded || !settings.autoUpdate) return;
    return untrack(() => updater.schedule());
  });

  // Dirty dots / missing state for every open repo: check new ones, drop closed ones.
  $effect(() => {
    if (!session.loaded) return;
    const paths = [...session.openRepos];
    untrack(() => {
      for (const p of [...health.dirty.keys(), ...health.missing]) if (!paths.includes(p)) health.forget(p);
      for (const p of paths) if (!health.dirty.has(p) && !health.missing.has(p)) health.check(p);
    });
  });

  // The selected repo's loaded state is the freshest source for its own dot.
  $effect(() => {
    if (repo.path && !repo.loading) health.setCount(repo.path, repo.state.files.length);
  });

  // Follow the selected repo: load it and watch it for changes.
  $effect(() => {
    const selected = session.selected;
    if (!session.loaded) return;
    const path = selected && !health.missing.has(selected) ? selected : null;
    untrack(() => repo.open(path));
  });

  // Watch every open repo (so dirty dots stay live); unwatch on close.
  let watched: string[] = [];
  $effect(() => {
    if (!session.loaded) return;
    const next = [...session.openRepos];
    untrack(() => {
      const { add, remove } = watchDiff(watched, next);
      for (const p of add) api.watchRepo(p).catch(() => {});
      for (const p of remove) api.unwatchRepo(p).catch(() => {});
      watched = next;
    });
  });

  function onKeydown(e: KeyboardEvent) {
    const action = matchShortcut(e);
    if (!action) return;
    if (action === 'escape') {
      if (ui.searchOpen || ui.settingsOpen) e.preventDefault();
      ui.searchOpen = false;
      ui.settingsOpen = false;
      return;
    }
    e.preventDefault();
    if (isGitAction(action)) {
      // Git actions never fire behind an open dialog or settings.
      if (modalState.current || ui.settingsOpen || !repo.path) return;
      const path = repo.path;
      if (action === 'stageAll' || action === 'unstageAll') {
        // During a merge/rebase, staging is how conflicts are marked resolved: leave that to the conflict panel.
        if (repo.state.operation) return;
        if (action === 'stageAll') runOp('Stage all', () => api.stageAll(path));
        else runOp('Unstage all', () => api.unstageAll(path));
      } else runCommand(action);
      return;
    }
    if (action === 'toggleRepos') session.reposCollapsed = !session.reposCollapsed;
    else if (action === 'search') {
      repo.showGraph();
      ui.searchOpen = true;
    }
    else if (action === 'settings') ui.settingsOpen = true;
    else if (action === 'openRepo') openRepoDialog();
    else if (action === 'closeRepo') {
      if (session.selected) session.closeRepo(session.selected);
    } else {
      const path = session.openRepos[action.repo - 1];
      if (path) session.selected = path;
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<main>
  <!-- The toolbar spans the whole window, so every column starts at the same height below it. -->
  {#if repo.path}
    <Toolbar />
  {:else}
    <div class="titlebar" data-tauri-drag-region></div>
  {/if}
  <Columns wide={!!repo.path && repo.centre.kind === 'diff'}>
    {#snippet repos()}
      <RepoColumn />
    {/snippet}
    {#snippet refs()}
      <RefsColumn />
    {/snippet}
    {#snippet centre()}
      {#if repo.path}
        <!-- GraphView stays mounted under the diff so its scroll position and page cache survive. -->
        <div class="centre">
          <ConflictBanner />
          <div class="stack">
            <div class="layer" class:covered={repo.centre.kind === 'diff'} inert={repo.centre.kind === 'diff'}>
              <GraphView />
            </div>
            {#if repo.centre.kind === 'diff'}
              <div class="layer"><DiffView /></div>
            {/if}
          </div>
        </div>
      {:else}
        <div class="ph">
          <p class="ph-title">No repository open</p>
          <p class="ph-hint">Open a local git repository to see its history.</p>
          <button class="primary" type="button" onclick={openRepoDialog}>Open repository… <span class="kbd">⌘O</span></button>
        </div>
      {/if}
    {/snippet}
    {#snippet details()}
      {#if repo.path && repo.state.operation}
        <ConflictPanel />
      {:else if repo.path && repo.selection?.kind === 'wip'}
        <WorkingChanges />
      {:else if repo.selection?.kind === 'commit'}
        <CommitDetails oid={repo.selection.oid} />
      {:else}
        <div class="ph">
          <p class="ph-title">No commit selected</p>
          <p class="ph-hint">Click a commit to see its message and changed files. Use ↑ and ↓ to move through history.</p>
        </div>
      {/if}
    {/snippet}
  </Columns>
</main>
{#if ui.settingsOpen}<Settings />{/if}
<Modal />
<Toasts />
<ContextMenu />

<style>
  main {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  /* With no repository open there is no toolbar, but the window still needs its title bar. */
  .titlebar {
    display: none;
    flex: none;
    height: var(--titlebar-h);
    border-bottom: 1px solid var(--line);
    background: var(--panel);
  }
  :global(:root[data-platform='mac']) .titlebar {
    display: block;
  }
  .centre {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .stack {
    position: relative;
    flex: 1;
    min-height: 0;
  }
  .layer {
    position: absolute;
    inset: 0;
  }
  .layer.covered {
    visibility: hidden;
    pointer-events: none;
  }
  .ph {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    max-width: 280px;
    margin: 0 auto;
    padding: 30vh 16px 0;
    text-align: center;
  }
  .ph p {
    margin: 0;
  }
  .ph-title {
    color: var(--text);
    font-weight: 600;
  }
  .ph-hint {
    color: var(--muted);
    font-size: var(--fs-sm);
    line-height: 1.5;
  }
  .ph button {
    margin-top: 8px;
  }
  .kbd {
    margin-left: 6px;
    opacity: 0.75;
  }
</style>
