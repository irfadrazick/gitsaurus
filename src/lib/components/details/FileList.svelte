<script lang="ts">
  import type { Snippet } from 'svelte';
  import { SvelteSet } from 'svelte/reactivity';
  import { session } from '../../state/session.svelte';
  import type { Change, ChangedFile, TreeNode } from '../../types';
  import { buildTree } from '../../util/tree';
  import type { WipFile } from '../../util/wip';

  let {
    files,
    onOpen,
    selected = null,
    actions,
    onContext,
    bare = false,
  }: {
    files: WipFile[];
    onOpen: (file: ChangedFile) => void;
    /** Path of the file to highlight. */
    selected?: string | null;
    /** Rendered at the right end of every file row (stage/unstage buttons). */
    actions?: Snippet<[ChangedFile]>;
    /** Right-click on a file row. */
    onContext?: (file: ChangedFile, e: MouseEvent) => void;
    /** Embedded in a scrolling parent: no count/layout bar, no own scrolling. */
    bare?: boolean;
  } = $props();

  const tree = $derived(buildTree(files));
  const collapsed = new SvelteSet<string>();

  const LETTER: Record<Change, string> = { added: 'A', modified: 'M', deleted: 'D', renamed: 'R', copied: 'C', typeChange: 'T' };
  const LABEL: Record<Change, string> = {
    added: 'Added', modified: 'Modified', deleted: 'Deleted', renamed: 'Renamed', copied: 'Copied', typeChange: 'Type changed',
  };

  const base = (p: string) => p.slice(p.lastIndexOf('/') + 1);
  const dir = (p: string) => (p.includes('/') ? p.slice(0, p.lastIndexOf('/') + 1) : '');

  function toggle(path: string) {
    if (!collapsed.delete(path)) collapsed.add(path);
  }
</script>

{#snippet row(file: WipFile, depth: number, name: string | null)}
  <div
    class="row"
    class:sel={selected === file.path}
    style:padding-left="{8 + depth * 14}px"
    role="presentation"
    oncontextmenu={onContext ? (e) => (e.preventDefault(), onContext(file, e)) : undefined}
  >
    <button class="open" type="button" title={file.origPath ? `${file.origPath} → ${file.path}` : file.path} onclick={() => onOpen(file)}>
      {#if file.conflict}
        <span class="badge conflict" title="Conflicted">U</span>
      {:else}
        <span class="badge {file.change}" title={LABEL[file.change]}>{LETTER[file.change]}</span>
      {/if}
      {#if name !== null}
        <span class="base">{name}</span>
      {:else}
        <span class="dir">{dir(file.path)}</span><span class="base">{base(file.path)}</span>
      {/if}
      {#if file.origPath}<span class="dir orig">← {file.origPath}</span>{/if}
    </button>
    {#if actions}<span class="actions">{@render actions(file)}</span>{/if}
  </div>
{/snippet}

{#snippet nodes(list: TreeNode[], depth: number)}
  {#each list as n (n.path)}
    {#if n.isDir}
      <button class="folder" type="button" style:padding-left="{8 + depth * 14}px" aria-expanded={!collapsed.has(n.path)} onclick={() => toggle(n.path)}>
        <span class="chev" class:closed={collapsed.has(n.path)}>▾</span>{n.name}
      </button>
      {#if !collapsed.has(n.path)}{@render nodes(n.children, depth + 1)}{/if}
    {:else if n.file}
      {@render row(n.file, depth, n.name)}
    {/if}
  {/each}
{/snippet}

<div class="filelist" class:bare>
  {#if !bare}<div class="bar">
    <span class="count muted small">{files.length} {files.length === 1 ? 'file' : 'files'}</span>
    <div class="seg" role="group" aria-label="File list layout">
      <button type="button" class:on={session.fileListMode === 'path'} aria-pressed={session.fileListMode === 'path'} onclick={() => (session.fileListMode = 'path')}>Path</button>
      <button type="button" class:on={session.fileListMode === 'tree'} aria-pressed={session.fileListMode === 'tree'} onclick={() => (session.fileListMode = 'tree')}>Tree</button>
    </div>
  </div>{/if}
  <div class="items">
    {#if session.fileListMode === 'tree'}
      {@render nodes(tree, 0)}
    {:else}
      {#each files as f (f.path)}
        {@render row(f, 0, null)}
      {/each}
    {/if}
  </div>
</div>

<style>
  .filelist {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
  }
  .filelist.bare {
    height: auto;
  }
  .bare .items {
    overflow: visible;
    padding: 0;
  }
  .bar {
    display: flex;
    flex: none;
    align-items: center;
    justify-content: space-between;
    padding: 6px 8px;
    border-bottom: 1px solid var(--line);
  }
  .seg {
    display: inline-flex;
  }
  .seg button {
    padding: 1px 8px;
    border-radius: 0;
    font-size: var(--font-min);
    background: transparent;
    color: var(--muted);
  }
  .seg button:first-child {
    border-radius: 5px 0 0 5px;
  }
  .seg button:last-child {
    border-radius: 0 5px 5px 0;
    border-left: 0;
  }
  .seg button.on {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }
  .items {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 4px 0;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 4px;
    min-height: 26px;
    padding-right: 6px;
  }
  .row:not(:hover):not(:focus-within) .actions {
    visibility: hidden;
  }
  .row:hover,
  .row.sel {
    background: color-mix(in srgb, var(--accent) 22%, transparent);
  }
  .open,
  .folder {
    display: flex;
    flex: 1;
    min-width: 0;
    align-items: center;
    gap: 6px;
    padding-top: 3px;
    padding-bottom: 3px;
    border: 0;
    border-radius: 0;
    background: transparent;
    text-align: left;
    white-space: nowrap;
    font-size: 12px;
  }
  .open {
    padding-left: 0;
  }
  .folder {
    width: 100%;
    color: var(--muted);
  }
  .folder:hover {
    background: color-mix(in srgb, var(--text) 6%, transparent);
  }
  .chev {
    display: inline-block;
    width: 10px;
    font-size: 10px;
  }
  .chev.closed {
    transform: rotate(-90deg);
  }
  .badge {
    flex: none;
    width: 16px;
    height: 16px;
    border-radius: 4px;
    text-align: center;
    font: 700 10px/16px var(--mono);
    color: #11151b;
    background: var(--lane-5);
  }
  .badge.conflict {
    background: var(--del);
    color: #fff;
  }
  .badge.added {
    background: var(--add);
  }
  .badge.deleted {
    background: var(--del);
  }
  .badge.renamed,
  .badge.copied {
    background: var(--lane-1);
  }
  .badge.typeChange {
    background: var(--lane-2);
  }
  .dir {
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--muted);
  }
  .dir.orig {
    flex: none;
    font-size: var(--font-min);
  }
  .base {
    flex: none;
    max-width: 100%;
  }
  .actions {
    display: inline-flex;
    flex: none;
    gap: 4px;
  }
</style>
