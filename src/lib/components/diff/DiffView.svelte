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
  import Icon from '../Icon.svelte';

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
  /** Where the diff comes from, so the header answers "which version am I looking at?". */
  const sourceLabel = $derived.by(() => {
    const s = centre?.source;
    if (!s) return '';
    if (s.kind === 'commit') return s.oid.slice(0, 7);
    return { unstaged: 'Unstaged', staged: 'Staged', untracked: 'Untracked', workingFile: 'Working file' }[s.kind];
  });
</script>

<svelte:window onkeydown={onKeydown} />

{#if centre}
  <div class="diffview">
    <header>
      <span class="icon"><Icon name="file" /></span>
      <span class="path" {title}>{title}</span>
      {#if sourceLabel}<span class="source" class:mono={centre.source.kind === 'commit'}>{sourceLabel}</span>{/if}
      <span class="grow"></span>
      <button class="iconbtn" type="button" title="Back to graph (Esc)" aria-label="Close diff" onclick={() => repo.showGraph()}><Icon name="close" /></button>
    </header>
    <div class="toolbar">
      <div class="seg" role="group" aria-label="View">
        <button type="button" class:on={centre.view === 'file'} aria-pressed={centre.view === 'file'} onclick={() => setView('file')}>File</button>
        <button type="button" class:on={centre.view === 'diff'} aria-pressed={centre.view === 'diff'} onclick={() => setView('diff')}>Diff</button>
      </div>
      <span class="grow"></span>
      <button class="iconbtn" type="button" title="Previous change ([)" aria-label="Previous change" onclick={() => stepTo(-1)}><Icon name="arrowUp" /></button>
      <button class="iconbtn" type="button" title="Next change (])" aria-label="Next change" onclick={() => stepTo(1)}><Icon name="arrowDown" /></button>
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
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: var(--fs-sm) var(--mono);
  }
  .source {
    flex: none;
    padding: 0 7px;
    border-radius: var(--radius-xs);
    background: var(--chip);
    color: var(--chip-text);
    font-size: var(--fs-xs);
    line-height: 18px;
  }
  .toolbar {
    display: flex;
    flex: none;
    align-items: center;
    gap: 4px;
    padding: 5px 8px 5px 12px;
    border-bottom: 1px solid var(--line);
  }
  .toolbar .seg:last-child {
    margin-left: 6px;
  }
  .grow {
    flex: 1;
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
    font-size: var(--fs-sm);
  }
  .note.err {
    color: var(--danger-text);
    text-align: left;
    white-space: pre-wrap;
    font-family: var(--mono);
    user-select: text;
    -webkit-user-select: text;
  }
</style>
