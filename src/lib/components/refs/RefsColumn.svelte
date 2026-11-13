<script lang="ts">
  import Icon, { type IconName } from '../Icon.svelte';
  import { performMenuAction, type ActionContext } from '../../state/actions';
  import { openMenu } from '../../state/contextMenu.svelte';
  import { runOp } from '../../state/ops';
  import { repo } from '../../state/repo.svelte';
  import * as api from '../../api';
  import { openWorktree, performWorktreeAction } from '../../state/worktrees';
  import type { Branch, RemoteBranch, Stash, Tag, Worktree } from '../../types';
  import type { MenuKind } from '../../util/actionTable';
  import { branchMenu, remoteBranchMenu, stashMenu, tagMenu, worktreeMenu, type MenuItem } from '../../util/menus';
  import { checkedOutElsewhere, worktreeLabel } from '../../util/worktrees';

  let filter = $state('');
  let collapsed = $state<Record<string, boolean>>({});

  const q = $derived(filter.trim().toLowerCase());
  const match = (s: string) => !q || s.toLowerCase().includes(q);

  const refs = $derived(repo.refs);
  const local = $derived(refs.local.filter((b) => match(b.name)));
  const remoteGroups = $derived(
    refs.remotes
      .map((r) => ({ remote: r, branches: refs.remote.filter((b) => b.remote === r && match(b.name)) }))
      .filter((g) => g.branches.length > 0),
  );
  const remoteCount = $derived(remoteGroups.reduce((n, g) => n + g.branches.length, 0));
  const tags = $derived(refs.tags.filter((t) => match(t.name)));
  const stashes = $derived(refs.stashes.filter((s) => match(s.message)));
  // Only the main worktree means none were added: the section would just repeat Local.
  const worktrees = $derived(
    refs.worktrees.length > 1 ? refs.worktrees.filter((w) => match(worktreeLabel(w)) || match(w.name)) : [],
  );
  // With one remote, its name is a redundant level: list its branches straight under Remote.
  const groupRemotes = $derived(refs.remotes.length > 1);
  const elsewhere = $derived(checkedOutElsewhere(refs.worktrees));

  const selectedOid = $derived(repo.selection?.kind === 'commit' ? repo.selection.oid : null);

  function ctxFor(menu: MenuKind, name: string, extra: Partial<ActionContext> = {}): ActionContext {
    return {
      menu,
      repo: repo.path ?? '',
      name,
      current: refs.head.branch,
      isHeadBranch: false,
      detached: refs.head.branch === null,
      hasUpstream: false,
      ...extra,
    };
  }

  function show(e: MouseEvent, ctx: ActionContext, items: (c: ActionContext) => MenuItem[]) {
    e.preventDefault();
    openMenu(e.clientX, e.clientY, items(ctx), (id) => performMenuAction(id, ctx));
  }

  const branchCtx = (b: Branch) => ctxFor('branch', b.name, { oid: b.oid, isHeadBranch: b.isHead, hasUpstream: !!b.upstream });
  const remoteCtx = (b: RemoteBranch) => ctxFor('remoteBranch', b.fullName, { oid: b.oid });
  const tagCtx = (t: Tag) => ctxFor('tag', t.name, { oid: t.oid });
  const stashCtx = (s: Stash) => ctxFor('stash', `stash@{${s.index}}`, { oid: s.oid, stashIndex: s.index });

  function checkout(b: Branch) {
    if (b.isHead || !repo.path) return;
    const path = repo.path;
    runOp('Checkout', () => api.checkout(path, b.name, 'local'));
  }

  function showWorktree(e: MouseEvent, w: Worktree) {
    e.preventDefault();
    const path = repo.path;
    if (path) openMenu(e.clientX, e.clientY, worktreeMenu(w), (id) => performWorktreeAction(id, path, w));
  }

  function branchTitle(b: Branch) {
    const base = b.upstream ? `${b.name} → ${b.upstream}${b.gone ? ' (gone)' : ''}` : b.name;
    const w = elsewhere.get(b.name);
    return w ? `${base}\nChecked out in worktree ${w.name}` : base;
  }

  const toggle = (k: string) => (collapsed[k] = !collapsed[k]);
  const open = (k: string) => !collapsed[k] || !!q;
</script>

<div class="col">
  <div class="filter">
    <input type="search" placeholder="Filter branches, tags…" bind:value={filter} aria-label="Filter refs" spellcheck="false" autocomplete="off" autocapitalize="off" />
  </div>
  <div class="scroll">
    {#if !repo.path}
      <p class="empty">No repository selected</p>
    {:else}
      {#snippet header(key: string, title: string, count: number, icon?: IconName)}
        <button class="sec" onclick={() => toggle(key)} aria-expanded={open(key)}>
          <span class="chev" class:closed={!open(key)}><Icon name="chevronDown" size={12} /></span>
          {#if icon}<Icon name={icon} size={14} />{/if}
          <span class="stitle">{title}</span>
          <span class="count" class:zero={count === 0}>{count}</span>
        </button>
      {/snippet}

      {#if !q || local.length}
        {@render header('local', 'Local', local.length, 'computer')}
        {#if open('local')}
          {#each local as b (b.name)}
            <div
              class="row"
              class:selected={selectedOid === b.oid}
              class:head={b.isHead}
              role="button"
              tabindex="0"
              title={branchTitle(b)}
              onclick={() => repo.revealCommit(b.oid)}
              onkeydown={(e) => e.key === 'Enter' && repo.revealCommit(b.oid)}
              ondblclick={() => checkout(b)}
              oncontextmenu={(e) => show(e, branchCtx(b), branchMenu)}
            >
              <span class="check">{#if b.isHead}<Icon name="check" size={12} stroke={2} />{/if}</span>
              <span class="name">{b.name}</span>
              {#if elsewhere.has(b.name)}<span class="wt" aria-label="Checked out in another worktree"><Icon name="folder" size={12} /></span>{/if}
              {#if b.gone}<span class="badge gone">gone</span>{/if}
              {#if b.ahead > 0}<span class="badge">↑{b.ahead}</span>{/if}
              {#if b.behind > 0}<span class="badge">↓{b.behind}</span>{/if}
            </div>
          {/each}
        {/if}
      {/if}

      {#if !q || remoteCount}
        {@render header('remote', 'Remote', remoteCount, 'cloud')}
        {#if open('remote')}
          {#each remoteGroups as g (g.remote)}
            {#if groupRemotes}<div class="remote-name">{g.remote}</div>{/if}
            {#each g.branches as b (b.fullName)}
              <div
                class="row"
                class:selected={selectedOid === b.oid}
                role="button"
                tabindex="0"
                title={b.fullName}
                onclick={() => repo.revealCommit(b.oid)}
                onkeydown={(e) => e.key === 'Enter' && repo.revealCommit(b.oid)}
                oncontextmenu={(e) => show(e, remoteCtx(b), remoteBranchMenu)}
              >
                <span class="check"></span>
                <span class="name">{b.name}</span>
              </div>
            {/each}
          {/each}
        {/if}
      {/if}

      {#if !q || tags.length}
        {@render header('tags', 'Tags', tags.length, 'tag')}
        {#if open('tags')}
          {#each tags as t (t.name)}
            <div
              class="row"
              class:selected={selectedOid === t.oid}
              role="button"
              tabindex="0"
              title={t.annotated ? `${t.name} (annotated)` : t.name}
              onclick={() => repo.revealCommit(t.oid)}
              onkeydown={(e) => e.key === 'Enter' && repo.revealCommit(t.oid)}
              oncontextmenu={(e) => show(e, tagCtx(t), tagMenu)}
            >
              <span class="check"></span>
              <span class="name">{t.name}</span>
            </div>
          {/each}
        {/if}
      {/if}

      {#if !q || stashes.length}
        {@render header('stashes', 'Stashes', stashes.length, 'stash')}
        {#if open('stashes')}
          {#each stashes as s (s.index)}
            <div
              class="row"
              role="button"
              tabindex="0"
              title={s.message}
              onclick={() => repo.selectStash(s.oid)}
              onkeydown={(e) => e.key === 'Enter' && repo.selectStash(s.oid)}
              oncontextmenu={(e) => show(e, stashCtx(s), stashMenu)}
            >
              <span class="check"></span>
              <span class="name">{s.message}</span>
            </div>
          {/each}
        {/if}
      {/if}

      {#if worktrees.length}
        {@render header('worktrees', 'Worktrees', worktrees.length, 'folder')}
        {#if open('worktrees')}
          {#each worktrees as w (w.path)}
            <div
              class="row"
              class:selected={!!w.oid && selectedOid === w.oid}
              class:head={w.isCurrent}
              class:missing={w.missing}
              role="button"
              tabindex="0"
              title={w.missing ? `${w.path} (folder missing)` : w.path}
              onclick={() => w.oid && repo.revealCommit(w.oid)}
              onkeydown={(e) => e.key === 'Enter' && w.oid && repo.revealCommit(w.oid)}
              ondblclick={() => openWorktree(w)}
              oncontextmenu={(e) => showWorktree(e, w)}
            >
              <span class="check">{#if w.isCurrent}<Icon name="check" size={12} stroke={2} />{/if}</span>
              <span class="name wtname" class:mono={!w.branch}>{worktreeLabel(w)}</span>
              {#if w.missing}<span class="badge gone">missing</span>{/if}
              <span class="folder">
                <Icon name={w.isMain ? 'home' : 'folder'} size={12} />
                <span class="fname">{w.name}</span>
              </span>
            </div>
          {/each}
        {/if}
      {/if}

      {#if q && !local.length && !remoteCount && !tags.length && !stashes.length && !worktrees.length}
        <p class="empty">No refs match “{filter}”</p>
      {/if}
    {/if}
  </div>
</div>

<style>
  .col {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .filter {
    padding: 8px;
    border-bottom: 1px solid var(--line);
  }
  .filter input {
    width: 100%;
    height: 28px;
  }
  .scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0 6px 12px;
  }
  /* Section headers: a full-width rule above each one, and they stay pinned while
     their list scrolls under them. */
  .sec {
    position: sticky;
    top: 0;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    width: calc(100% + 12px);
    height: 32px;
    margin: 6px -6px 2px;
    padding: 0 12px;
    border: 0;
    border-top: 1px solid var(--line);
    border-radius: 0;
    background: var(--panel);
    color: var(--muted);
    font-size: var(--fs-xs);
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    text-align: left;
  }
  .sec:first-child {
    margin-top: 0;
    border-top-color: transparent;
  }
  .sec:hover:not(:disabled) {
    color: var(--text);
    border: 0;
    border-top: 1px solid var(--line);
    background: color-mix(in srgb, var(--text) 5%, var(--panel));
  }
  .sec:first-child:hover:not(:disabled) {
    border-top-color: transparent;
  }
  .sec:focus-visible {
    outline: 2px solid var(--focus);
    outline-offset: -2px;
  }
  .chev {
    display: inline-flex;
    width: 12px;
    transition: transform 0.12s ease-out;
  }
  .chev.closed {
    transform: rotate(-90deg);
  }
  .stitle {
    flex: 1;
  }
  .count {
    font-weight: 500;
    letter-spacing: 0;
    font-variant-numeric: tabular-nums;
  }
  .count.zero {
    opacity: 0.5;
  }
  .remote-name {
    height: 24px;
    padding: 4px 10px 0;
    font-size: var(--fs-xs);
    color: var(--muted);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 8px;
    border-radius: 0;
    cursor: default;
  }
  .row:hover {
    background: var(--hover);
  }
  .row.selected {
    background: var(--sel);
    color: var(--sel-text);
  }
  .row:focus-visible {
    outline: 2px solid var(--focus);
    outline-offset: -2px;
  }
  .row.head .name {
    font-weight: 600;
  }
  .check {
    flex: none;
    width: 12px;
    color: var(--add);
  }
  .name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .badge {
    flex: none;
    padding: 0 6px;
    border-radius: var(--radius-lg);
    background: var(--chip);
    color: var(--chip-text);
    font-size: var(--fs-xs);
    line-height: 16px;
    font-variant-numeric: tabular-nums;
  }
  .badge.gone {
    color: var(--danger-text);
  }
  .row.selected .badge {
    background: rgb(255 255 255 / 0.12);
    color: var(--sel-text);
  }
  .wt {
    flex: none;
    display: inline-flex;
    color: var(--muted);
  }
  .row.selected .wt,
  .row.selected .folder {
    color: var(--sel-muted);
  }
  /* The branch keeps its width; the folder name gives way first. */
  .name.wtname {
    flex: 1 1 auto;
  }
  .folder {
    flex: 0 6 auto;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    min-width: 20px;
    color: var(--muted);
    font-size: var(--fs-sm);
  }
  .fname {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .mono {
    font-family: var(--mono);
    font-size: var(--fs-sm);
  }
  .row.missing .name {
    color: var(--muted);
    text-decoration: line-through;
  }
  .empty {
    padding: 12px;
    color: var(--muted);
    font-size: var(--fs-sm);
  }
</style>
