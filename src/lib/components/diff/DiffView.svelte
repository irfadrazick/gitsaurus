<script lang="ts">
  import { tick, untrack } from 'svelte';
  import * as api from '../../api';
  import { modalState } from '../../state/modal.svelte';
  import { repo } from '../../state/repo.svelte';
  import { session } from '../../state/session.svelte';
  import { ui } from '../../state/ui.svelte';
  import type { DiffLine, FileDiff } from '../../types';
  import { shouldCloseDiffOnEsc } from '../../util/diffKeys';
  import { stepHunk } from '../../util/hunkNav';
  import { isLiveSource } from '../../util/liveDiff';
  import { highlightLines } from './highlight';
  import SplitDiff from './SplitDiff.svelte';
  import UnifiedDiff from './UnifiedDiff.svelte';

  const FILE_VIEW_CONTEXT = 1_000_000;
  const DIFF_VIEW_CONTEXT = 3;

  const centre = $derived(repo.centre.kind === 'diff' ? repo.centre : null);

  let diff = $state.raw<FileDiff | null>(null);
  let html = $state.raw<ReadonlyMap<DiffLine, string>>(new Map());
  let error = $state<string | null>(null);
  let loading = $state(false);
  let body = $state<HTMLDivElement>();
  let token = 0;
  let shownKey = '';

  // Load the diff whenever the file, source or view changes, and, for working-state sources,
  // whenever the repo state refreshes. Stale responses are dropped by `token`.
  $effect(() => {
    const path = repo.path;
    const c = centre;
    if (!path || !c) return;
    // Read stateVersion only for live sources, so commit diffs don't re-fetch on refresh.
    if (isLiveSource(c.source)) void repo.stateVersion;
    const key = JSON.stringify([path, c.path, c.origPath, c.source, c.view]);
    const t = ++token;
    const context = c.view === 'file' ? FILE_VIEW_CONTEXT : DIFF_VIEW_CONTEXT;
    // A live refresh of the file already shown keeps scroll position and shows no "Loading…".
    const refresh = untrack(() => key === shownKey && diff !== null);
    if (!refresh) loading = true;
    error = null;
    api.fileDiff(path, c.path, c.origPath, c.source, context).then(
      async (d) => {
        if (t !== token) return;
        if (refresh && diff && JSON.stringify(d) === JSON.stringify(diff)) return;
        diff = d;
        html = new Map();
        loading = false;
        shownKey = key;
        await tick();
        if (!refresh && t === token && body) body.scrollTop = 0;
        highlight(d, t);
      },
      (e) => {
        if (t !== token) return;
        diff = null;
        shownKey = '';
        loading = false;
        error = (e as { stderr?: string; message?: string })?.stderr || (e as { message?: string })?.message || String(e);
      },
    );
  });

  /** Highlight old and new sides of each hunk as joined blocks, then map results back to lines by position. */
  async function highlight(d: FileDiff, t: number) {
    if (d.binary || d.tooLarge) return;
    const out = new Map<DiffLine, string>();
    for (const h of d.hunks) {
      const oldSide = h.lines.filter((l) => l.kind === 'context' || l.kind === 'del');
      const newSide = h.lines.filter((l) => l.kind === 'context' || l.kind === 'add');
      const [oldHtml, newHtml] = await Promise.all([
        highlightLines(oldSide.map((l) => l.text), d.path),
        highlightLines(newSide.map((l) => l.text), d.path),
      ]);
      if (t !== token) return;
      oldSide.forEach((l, i) => l.kind === 'del' && out.set(l, oldHtml[i]));
      newSide.forEach((l, i) => out.set(l, newHtml[i]));
    }
    if (t === token) html = out;
  }

  function setView(view: 'diff' | 'file') {
    if (centre && centre.view !== view) repo.centre = { ...centre, view };
  }

  function stepTo(dir: 1 | -1) {
    if (!body) return;
    const heads = [...body.querySelectorAll<HTMLElement>('[data-hunk]')];
    const base = body.getBoundingClientRect().top - body.scrollTop;
    const tops = heads.map((h) => h.getBoundingClientRect().top - base);
    const i = stepHunk(tops, body.scrollTop, dir);
    if (i !== null) body.scrollTo({ top: tops[i] });
  }

  function onKeydown(e: KeyboardEvent) {
    if (!centre || e.metaKey || e.ctrlKey || e.altKey) return;
    const el = e.target as HTMLElement | null;
    if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.isContentEditable)) return;
    if (e.key === 'Escape') {
      // Layers that consume Esc call preventDefault; the state checks cover listener-order races.
      if (shouldCloseDiffOnEsc(e) && !modalState.current && !ui.settingsOpen) repo.showGraph();
    } else if (e.key === '[') stepTo(-1);
    else if (e.key === ']') stepTo(1);
  }

  const title = $derived(centre?.origPath ? `${centre.origPath} → ${centre.path}` : (centre?.path ?? ''));
</script>

<svelte:window onkeydown={onKeydown} />

{#if centre}
  <div class="diffview">
    <header>
      <svg class="icon" viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
        <path d="M4 1.5h5l3.5 3.5v9.5h-8.5z M9 1.5v3.5h3.5" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
      </svg>
      <span class="path" {title}>{title}</span>
      <button class="x" type="button" title="Back to graph (Esc)" aria-label="Close diff" onclick={() => repo.showGraph()}>×</button>
    </header>
    <div class="toolbar">
      <div class="seg" role="group" aria-label="View">
        <button type="button" class:on={centre.view === 'file'} aria-pressed={centre.view === 'file'} onclick={() => setView('file')}>File View</button>
        <button type="button" class:on={centre.view === 'diff'} aria-pressed={centre.view === 'diff'} onclick={() => setView('diff')}>Diff View</button>
      </div>
      <span class="grow"></span>
      <button type="button" title="Previous hunk ([)" aria-label="Previous hunk" onclick={() => stepTo(-1)}>↑</button>
      <button type="button" title="Next hunk (])" aria-label="Next hunk" onclick={() => stepTo(1)}>↓</button>
      <div class="seg" role="group" aria-label="Layout">
        <button type="button" class:on={session.diffMode === 'unified'} aria-pressed={session.diffMode === 'unified'} onclick={() => (session.diffMode = 'unified')}>Unified</button>
        <button type="button" class:on={session.diffMode === 'split'} aria-pressed={session.diffMode === 'split'} onclick={() => (session.diffMode = 'split')}>Split</button>
      </div>
    </div>
    <div class="body" bind:this={body}>
      {#if error}
        <div class="note err">{error}</div>
      {:else if !diff}
        <div class="note">{loading ? 'Loading…' : ''}</div>
      {:else if diff.binary}
        <div class="note">Binary file not shown</div>
      {:else if diff.tooLarge}
        <div class="note">File too large to display (over 1 MB or 20,000 lines)</div>
      {:else if diff.hunks.length === 0}
        <div class="note">No changes to display</div>
      {:else if session.diffMode === 'split'}
        <SplitDiff hunks={diff.hunks} {html} />
      {:else}
        <UnifiedDiff hunks={diff.hunks} {html} />
      {/if}
    </div>
  </div>
{/if}

<style>
  .diffview {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    background: var(--bg);
  }
  header {
    display: flex;
    flex: none;
    align-items: center;
    gap: 8px;
    height: 34px;
    padding: 0 8px 0 12px;
    border-bottom: 1px solid var(--line);
  }
  .icon {
    flex: none;
    color: var(--muted);
  }
  .path {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: 12px var(--mono);
  }
  .x {
    padding: 0 8px;
    border: 0;
    background: transparent;
    font-size: 18px;
    line-height: 1;
    color: var(--muted);
  }
  .x:hover {
    color: var(--text);
  }
  .toolbar {
    display: flex;
    flex: none;
    align-items: center;
    gap: 6px;
    padding: 5px 12px;
    border-bottom: 1px solid var(--line);
  }
  .toolbar button {
    padding: 2px 9px;
    font-size: 12px;
  }
  .grow {
    flex: 1;
  }
  .seg {
    display: inline-flex;
  }
  .seg button {
    border-radius: 0;
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
  .body {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
  .note {
    padding: 24px 16px;
    text-align: center;
    color: var(--muted);
    font-size: 12px;
  }
  .note.err {
    color: var(--del);
  }
</style>
