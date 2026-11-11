<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { onMount, untrack } from 'svelte';
  import * as api from './lib/api';
  import Columns from './lib/components/layout/Columns.svelte';
  import Modal from './lib/components/layout/Modal.svelte';
  import Toasts from './lib/components/layout/Toasts.svelte';
  import { repo } from './lib/state/repo.svelte';
  import { openRepoDialog } from './lib/state/repos';
  import { session } from './lib/state/session.svelte';
  import { settings } from './lib/state/settings.svelte';
  import { ui } from './lib/state/ui.svelte';
  import type { ChangeKind, RepoChanged } from './lib/types';
  import { matchShortcut } from './lib/util/shortcuts';

  onMount(() => {
    session.init();
    settings.init();
    let unlisten: (() => void) | undefined;
    listen<RepoChanged>('repo-changed', (ev) => {
      if (ev.payload.repo !== repo.path) return;
      const kinds: ChangeKind[] = ev.payload.kinds.map((k) => (k === 'git' ? 'Git' : 'Worktree'));
      repo.refresh(kinds);
    })
      .then((u) => (unlisten = u))
      .catch(() => {});
    return () => unlisten?.();
  });

  // Follow the selected repo: load it and watch it for changes.
  $effect(() => {
    const path = session.selected;
    if (!session.loaded) return;
    untrack(() => repo.open(path));
    if (!path) return;
    api.watchRepo(path).catch(() => {});
    return () => {
      api.unwatchRepo(path).catch(() => {});
    };
  });

  function onKeydown(e: KeyboardEvent) {
    const action = matchShortcut(e);
    if (!action) return;
    if (action === 'escape') {
      ui.searchOpen = false;
      ui.settingsOpen = false;
      return;
    }
    e.preventDefault();
    if (action === 'toggleRepos') session.reposCollapsed = !session.reposCollapsed;
    else if (action === 'search') ui.searchOpen = true;
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
  <Columns>
    {#snippet repos()}
      <div class="ph">{session.reposCollapsed ? '' : 'Repositories'}</div>
    {/snippet}
    {#snippet refs()}
      <div class="ph">Branches</div>
    {/snippet}
    {#snippet centre()}
      <div class="ph">{repo.path ?? 'Open a repository with ⌘O'}</div>
    {/snippet}
    {#snippet details()}
      <div class="ph">Details</div>
    {/snippet}
  </Columns>
</main>
<Modal />
<Toasts />

<style>
  main {
    height: 100%;
  }
  .ph {
    padding: 10px 12px;
    color: var(--muted);
    font-size: 12px;
  }
</style>
