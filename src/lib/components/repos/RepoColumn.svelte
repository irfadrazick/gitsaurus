<script lang="ts">
  import * as api from '../../api';
  import { openMenu } from '../../state/contextMenu.svelte';
  import { health } from '../../state/repoHealth.svelte';
  import { openRepoDialog } from '../../state/repos';
  import { session } from '../../state/session.svelte';
  import { repoInitials, repoName } from '../../util/repoInitials';
  import Icon from '../Icon.svelte';

  const toggleCollapsed = () => (session.reposCollapsed = !session.reposCollapsed);

  let dragFrom = $state<number | null>(null);
  let dragOver = $state<number | null>(null);

  function select(path: string) {
    session.selected = path;
  }

  function menu(e: MouseEvent, path: string) {
    e.preventDefault();
    const items = health.missing.has(path)
      ? [{ id: 'close', label: 'Close' }]
      : [
          { id: 'reveal', label: 'Reveal in Finder' },
          { id: 'close', label: 'Close', separatorBefore: true },
        ];
    openMenu(e.clientX, e.clientY, items, (id) => {
      if (id === 'close') session.closeRepo(path);
      else if (id === 'reveal') api.revealInFinder(path, path).catch(() => {});
    });
  }

  function drop(e: DragEvent, to: number) {
    e.preventDefault();
    const from = dragFrom;
    dragFrom = dragOver = null;
    if (from === null || from === to) return;
    const [moved] = session.openRepos.splice(from, 1);
    session.openRepos.splice(to, 0, moved);
  }

  function onKey(e: KeyboardEvent, path: string) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      select(path);
    }
  }
</script>

<div class="col" class:collapsed={session.reposCollapsed}>
  {#if !session.reposCollapsed}
    <header>
      <span class="title">Repositories</span>
      <span class="count">{session.openRepos.length}</span>
      <button class="iconbtn" onclick={toggleCollapsed} title="Collapse (⌘B)" aria-label="Collapse repositories"><Icon name="sidebar" /></button>
    </header>
  {/if}
  <div class="list" role="listbox" aria-label="Open repositories">
    {#each session.openRepos as path, i (path)}
      {@const name = repoName(path)}
      {@const missing = health.missing.has(path)}
      {@const dirty = (health.dirty.get(path) ?? 0) > 0}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="row"
        class:selected={session.selected === path}
        class:missing
        class:dragover={dragOver === i && dragFrom !== i}
        role="option"
        aria-selected={session.selected === path}
        tabindex="0"
        draggable="true"
        title={session.reposCollapsed ? (missing ? `${name} (missing)` : name) : path}
        onclick={() => select(path)}
        onkeydown={(e) => onKey(e, path)}
        oncontextmenu={(e) => menu(e, path)}
        ondragstart={(e) => {
          dragFrom = i;
          e.dataTransfer?.setData('text/plain', path);
          if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
        }}
        ondragover={(e) => {
          if (dragFrom === null) return;
          e.preventDefault();
          dragOver = i;
        }}
        ondrop={(e) => drop(e, i)}
        ondragend={() => (dragFrom = dragOver = null)}
      >
        {#if session.reposCollapsed}
          <span class="badge">{repoInitials(name)}</span>
        {:else}
          <span class="name">{name}</span>
          {#if missing}<span class="tag">missing</span>{/if}
        {/if}
        {#if dirty && !missing}<span class="dot" title="Uncommitted changes" aria-label="Uncommitted changes"></span>{/if}
      </div>
    {/each}
  </div>
  {#if session.reposCollapsed}
    <button class="iconbtn expand" onclick={toggleCollapsed} title="Expand (⌘B)" aria-label="Expand repositories"><Icon name="sidebar" /></button>
  {/if}
  <button class="open" onclick={openRepoDialog} title="Open repo… (⌘O)" aria-label="Open repo">
    <Icon name="plus" size={12} stroke={1.8} />{#if !session.reposCollapsed}<span>Open repo…</span>{/if}
  </button>
</div>

<style>
  .col {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    padding: 8px 6px 6px 12px;
    font-size: var(--fs-xs);
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .title {
    flex: 1;
  }
  .count {
    font-weight: 500;
  }
  .iconbtn {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--muted);
  }
  .iconbtn:hover:not(:disabled) {
    background: var(--hover);
    color: var(--text);
  }
  .iconbtn.expand {
    margin: 0 auto 2px;
  }
  .list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 2px 6px;
  }
  .row {
    position: relative;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 28px;
    padding: 0 10px;
    border-radius: 0;
    cursor: default;
    color: var(--text);
  }
  .row:hover {
    background: var(--hover);
  }
  .row.selected {
    background: var(--sel);
    color: var(--sel-text);
    font-weight: 600;
  }
  .row.missing {
    color: var(--muted);
    font-style: italic;
  }
  .row.dragover {
    box-shadow: 0 -2px 0 0 var(--focus);
  }
  .row:focus-visible {
    outline: 2px solid var(--focus);
    outline-offset: -2px;
  }
  .name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .tag {
    font-size: var(--fs-xs);
    padding: 0 6px;
    border-radius: var(--radius-lg);
    border: 1px solid var(--line);
    color: var(--muted);
    font-style: normal;
  }
  .dot {
    flex: none;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--lane-5);
  }
  .collapsed .row {
    justify-content: center;
    padding: 0;
    height: 32px;
    margin: 2px 0;
  }
  .collapsed .list {
    padding: 6px 4px;
  }
  .collapsed .dot {
    position: absolute;
    top: 4px;
    right: 4px;
  }
  .badge {
    display: grid;
    place-items: center;
    width: 30px;
    height: 26px;
    border-radius: var(--radius-xs);
    font-size: var(--fs-xs);
    font-weight: 600;
    letter-spacing: 0.02em;
  }
  .row:not(.selected) .badge {
    background: var(--chip);
  }
  .open {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    margin: 6px;
    height: 28px;
    padding: 0 8px;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--muted);
  }
  .open:hover:not(:disabled) {
    color: var(--text);
  }
</style>
