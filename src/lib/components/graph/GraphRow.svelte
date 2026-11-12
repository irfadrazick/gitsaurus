<script lang="ts">
  import type { GraphCommit } from '../../types';
  import { laneColor } from '../../util/graphGeometry';
  import { rowPills } from '../../util/refs';
  import GraphCell from './GraphCell.svelte';
  import RefPills from './RefPills.svelte';

  // Presentational only: GraphView handles clicks and context menus by delegation on `data-index` / `data-wip`.
  let {
    index,
    commit,
    wipCount,
    wipLinked = false,
    selected = false,
    match = false,
    current = false,
    marker = null,
    headIndex = null,
    hasWip = false,
  }: {
    index: number;
    /** Undefined while its page loads (skeleton row). */
    commit?: GraphCommit;
    /** Render the "// WIP" row with this many changed files. */
    wipCount?: number;
    wipLinked?: boolean;
    selected?: boolean;
    /** Search hit / the current search hit. */
    match?: boolean;
    current?: boolean;
    /** Relative-time chip text, when this row starts a new age bucket. */
    marker?: string | null;
    /** HEAD's graph index and whether a WIP row is shown: GraphCell styles the seed segments (R8). */
    headIndex?: number | null;
    hasWip?: boolean;
  } = $props();

  const isWip = $derived(wipCount !== undefined);
  const color = $derived(commit?.color ?? 0);
  const pills = $derived(commit ? rowPills(commit.refs) : []);
  // Skeleton bars vary in width so a loading page doesn't look like a grid.
  const skel = $derived(28 + ((index * 37) % 41));
</script>

{#if isWip}
  <div
    class="row wip"
    class:selected
    role="option"
    aria-selected={selected}
    data-wip=""
  >
    <div class="refs"></div>
    <div class="graph"><GraphCell wip wipLinked={wipLinked} selected={selected} /></div>
    <div class="msg">
      <span class="text"><span class="wiplabel">WIP</span><span class="body">{wipCount} changed {wipCount === 1 ? 'file' : 'files'}</span></span>
    </div>
  </div>
{:else if commit}
  <div
    class="row"
    class:selected
    class:match
    class:current
    role="option"
    aria-selected={selected}
    data-index={index}
  >
    <div class="refs"><RefPills pills={pills} color={laneColor(color)} /></div>
    <div class="graph">
      <GraphCell commit={commit} selected={selected} hasRefs={pills.length > 0} index={index} headIndex={headIndex} hasWip={hasWip} />
    </div>
    <div class="msg">
      <span class="text"><span class="summary">{commit.summary}</span>{#if commit.bodyPreview}<span class="body">{commit.bodyPreview}</span>{/if}</span>
      {#if marker}<span class="marker">{marker}</span>{/if}
    </div>
  </div>
{:else}
  <div class="row skeleton" role="option" aria-selected="false" aria-busy="true">
    <div class="refs"></div>
    <div class="graph"><span class="dot"></span></div>
    <div class="msg"><span class="bar" style:width="{skel}%"></span></div>
  </div>
{/if}

<style>
  /* Rows stay neutral: lane colour lives in the graph and the pills, so selection is the one strong fill. */
  .row {
    display: grid;
    grid-template-columns: var(--refs-w) var(--graph-w) minmax(0, 1fr);
    height: 36px;
    cursor: default;
  }
  .row:hover {
    background: var(--hover);
  }
  .row.selected {
    background: var(--sel);
  }
  .refs {
    min-width: 0;
    overflow: hidden;
  }
  .graph {
    overflow: hidden;
  }
  .msg {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
    padding: 0 12px 0 6px;
  }
  .text {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .summary {
    color: var(--text);
  }
  .row.selected .summary {
    color: var(--sel-text);
  }
  .body {
    margin-left: 8px;
    color: var(--muted);
    font-size: var(--fs-sm);
  }
  .row.selected .body,
  .row.selected .marker {
    color: var(--sel-muted);
  }
  .wiplabel {
    color: var(--muted);
    font-size: var(--fs-sm);
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .wip .body {
    color: var(--text);
    font-size: var(--fs-md);
  }
  .marker {
    flex: none;
    padding: 0 7px;
    border-radius: 9px;
    background: var(--chip);
    color: var(--chip-text);
    font-size: var(--fs-xs);
    line-height: 18px;
    white-space: nowrap;
  }
  /* Search hits: an amber edge on the message cell; the current hit is stronger. */
  .row.match .msg {
    box-shadow: inset 3px 0 0 color-mix(in srgb, var(--match) 55%, transparent);
  }
  .row.match .summary {
    color: color-mix(in srgb, var(--match) 70%, #fff);
  }
  .row.current .msg {
    box-shadow: inset 3px 0 0 var(--match), inset 0 0 0 1px color-mix(in srgb, var(--match) 45%, transparent);
  }
  .skeleton:hover {
    background: transparent;
  }
  .dot {
    display: block;
    width: 10px;
    height: 10px;
    margin: 13px 0 0 9px;
    border-radius: 50%;
    background: rgb(255 255 255 / 0.06);
  }
  .bar {
    height: 10px;
    border-radius: var(--radius-sm);
    background: linear-gradient(90deg, rgb(255 255 255 / 0.05), rgb(255 255 255 / 0.09), rgb(255 255 255 / 0.05));
    background-size: 200% 100%;
    animation: shimmer 1.4s linear infinite;
  }
  @keyframes shimmer {
    from {
      background-position: 100% 0;
    }
    to {
      background-position: -100% 0;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .bar {
      animation: none;
    }
  }
</style>
