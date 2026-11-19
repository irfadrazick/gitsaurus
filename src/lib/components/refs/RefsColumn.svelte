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
  import { refTree, type RefNode } from '../../util/refs';
  import { checkedOutElsewhere, worktreeLabel } from '../../util/worktrees';

  let filter = $state('');
  let collapsed = $state<Record<string, boolean>>({});

  const q = $derived(filter.trim().toLowerCase());
  const match = (s: string) => !q || s.toLowerCase().includes(q);

  const refs = $derived(repo.refs);
  const local = $derived(refs.local.filter((b) => match(b.name)));
  // Branch names with slashes are shown as folders: `release/v4.7` is `v4.7` inside `release`.
  const localTree = $derived(refTree(local, (b) => b.name));
  const remoteGroups = $derived(
    refs.remotes
      .map((r) => ({ remote: r, branches: refs.remote.filter((b) => b.remote === r && match(b.name)) }))
      .filter((g) => g.branches.length > 0)
      .map((g) => ({ ...g, tree: refTree(g.branches, (b) => b.name) })),
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

  /** Checks out the local branch of the same name, creating it to track `b` if there is none. */
  function checkoutRemote(b: RemoteBranch) {
    if (!repo.path || refs.local.some((l) => l.isHead && l.name === b.name)) return;
    const path = repo.path;
    runOp('Checkout', () => api.checkout(path, b.fullName, 'remote'));
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

  /** Indent per folder level: a folder's contents line up with its name, past the chevron and icon. */
  const INDENT = 18;
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
    {:else if repo.loading && !local.length && !remoteCount && !tags.length}
      <!-- A repository is opening: placeholder sections, shown only if that takes a moment. -->
      <div class="skeleton-delayed" aria-hidden="true">
        {#each [3, 2, 4] as rows, g (g)}
          <div class="skhead"><span class="skeleton-bar"></span></div>
          {#each { length: rows } as _, i (i)}
            <div class="skrow"><span class="skeleton-bar" style:width="{36 + ((g * 3 + i) * 23) % 40}%"></span></div>
          {/each}
        {/each}
      </div>
    {:else}
      {#snippet header(key: string, title: string, count: number, icon?: IconName)}
        <button class="sec" onclick={() => toggle(key)} aria-expanded={open(key)}>
          <span class="chev" class:closed={!open(key)}><Icon name="chevronDown" size={12} /></span>
          {#if icon}<Icon name={icon} size={14} />{/if}
          <span class="stitle">{title}</span>
          <span class="count" class:zero={count === 0}>{count}</span>
        </button>
      {/snippet}

      {#snippet folder(key: string, name: string, depth: number)}
        <div
          class="row"
          style:padding-left="{8 + depth * INDENT}px"
          role="button"
          tabindex="0"
          aria-expanded={open(key)}
          onclick={() => toggle(key)}
          onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && (e.preventDefault(), toggle(key))}
        >
          <span class="check chev" class:closed={!open(key)}><Icon name="chevronDown" size={12} /></span>
          <span class="ficon"><Icon name="folder" size={12} /></span>
          <span class="name">{name}</span>
        </div>
      {/snippet}

      {#snippet localNodes(nodes: RefNode<Branch>[], depth: number)}
        {#each nodes as n (n.kind === 'folder' ? `/${n.path}` : n.item.name)}
          {#if n.kind === 'folder'}
            {@const key = `local/${n.path}`}
            {@render folder(key, n.name, depth)}
            {#if open(key)}{@render localNodes(n.children, depth + 1)}{/if}
          {:else}
            {@const b = n.item}
            <div
              class="row"
              class:selected={selectedOid === b.oid}
              class:head={b.isHead}
              style:padding-left="{8 + depth * INDENT}px"
              role="button"
              tabindex="0"
              title={branchTitle(b)}
              onclick={() => repo.revealCommit(b.oid)}
              onkeydown={(e) => e.key === 'Enter' && repo.revealCommit(b.oid)}
              ondblclick={() => checkout(b)}
              oncontextmenu={(e) => show(e, branchCtx(b), branchMenu)}
            >
              <span class="check">{#if b.isHead}<Icon name="check" size={12} stroke={2} />{/if}</span>
              <span class="name">{n.name}</span>
              {#if elsewhere.has(b.name)}<span class="wt" aria-label="Checked out in another worktree"><Icon name="folder" size={12} /></span>{/if}
              {#if b.gone}<span class="badge gone">gone</span>{/if}
              {#if b.ahead > 0}<span class="badge">↑{b.ahead}</span>{/if}
              {#if b.behind > 0}<span class="badge">↓{b.behind}</span>{/if}
            </div>
          {/if}
        {/each}
      {/snippet}

      {#snippet remoteNodes(nodes: RefNode<RemoteBranch>[], depth: number, remote: string)}
        {#each nodes as n (n.kind === 'folder' ? `/${n.path}` : n.item.fullName)}
          {#if n.kind === 'folder'}
            {@const key = `remote/${remote}/${n.path}`}
            {@render folder(key, n.name, depth)}
            {#if open(key)}{@render remoteNodes(n.children, depth + 1, remote)}{/if}
          {:else}
            {@const b = n.item}
            <div
              class="row"
              class:selected={selectedOid === b.oid}
              style:padding-left="{8 + depth * INDENT}px"
              role="button"
              tabindex="0"
              title={b.fullName}
              onclick={() => repo.revealCommit(b.oid)}
              onkeydown={(e) => e.key === 'Enter' && repo.revealCommit(b.oid)}
              ondblclick={() => checkoutRemote(b)}
              oncontextmenu={(e) => show(e, remoteCtx(b), remoteBranchMenu)}
            >
              <span class="check"></span>
              <span class="name">{n.name}</span>
            </div>
          {/if}
        {/each}
      {/snippet}

      {#if !q || local.length}
        {@render header('local', 'Local', local.length, 'computer')}
        {#if open('local')}
          {@render localNodes(localTree, 0)}
        {/if}
      {/if}

      {#if !q || remoteCount}
        {@render header('remote', 'Remote', remoteCount, 'cloud')}
        {#if open('remote')}
          {#each remoteGroups as g (g.remote)}
            {#if groupRemotes}<div class="remote-name">{g.remote}</div>{/if}
            {@render remoteNodes(g.tree, 0, g.remote)}
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
  .row .chev {
    color: var(--muted);
  }
  .ficon {
    flex: none;
    display: inline-flex;
    color: var(--muted);
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
  /* Placeholders sized like a section header and a row. */
  .skhead {
    display: flex;
    align-items: center;
    height: 32px;
    margin: 6px 0 2px;
    padding: 0 6px;
    border-top: 1px solid var(--line);
  }
  .skhead .skeleton-bar {
    width: 72px;
    height: 8px;
  }
  .skrow {
    display: flex;
    align-items: center;
    height: 28px;
    padding: 0 8px 0 26px; /* past the 12 px check column and its 6 px gap */
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
