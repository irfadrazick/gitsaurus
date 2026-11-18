<script lang="ts">
  import type { GraphCommit } from '../../types';
  import { formatDate } from '../../util/date';
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
    age = '',
    headIndex = null,
    hasWip = false,
    pillRoom = 0,
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
    /** Compact author age shown at the right edge, e.g. "10h". */
    age?: string;
    /** HEAD's graph index and whether a WIP row is shown: GraphCell styles the seed segments (R8). */
    headIndex?: number | null;
    hasWip?: boolean;
    /** Width the ref pills may take, in px (0 while unmeasured). */
    pillRoom?: number;
  } = $props();

  const isWip = $derived(wipCount !== undefined);
  const color = $derived(commit?.color ?? 0);
  const pills = $derived(commit ? rowPills(commit.refs) : []);
  // Skeleton bars vary in width so a loading page doesn't look like a grid.
  const skel = $derived(28 + ((index * 61 + 17) % 43));
</script>

{#if isWip}
  <div
    class="row wip"
    class:selected
    role="option"
    aria-selected={selected}
    data-wip=""
  >
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
    <div class="graph">
      <GraphCell commit={commit} selected={selected} index={index} headIndex={headIndex} hasWip={hasWip} />
    </div>
    <div class="msg">
      {#if pills.length}<RefPills pills={pills} color={laneColor(color)} room={pillRoom} />{/if}
      <span class="text"><span class="summary">{commit.summary}</span>{#if commit.bodyPreview}<span class="body">{commit.bodyPreview}</span>{/if}</span>
      <span class="age" title={formatDate(commit.authorTime)}>{age}</span>
    </div>
  </div>
{:else}
  <div class="row skeleton" role="option" aria-selected="false" aria-busy="true">
    <div class="graph"><span class="dot"></span></div>
    <div class="msg"><span class="skeleton-bar" style:width="{skel}%"></span></div>
  </div>
{/if}

<style>
  /* Rows stay neutral: lane colour lives in the graph and the pills, so selection is the one strong fill. */
  .row {
    display: grid;
    grid-template-columns: var(--graph-w) minmax(0, 1fr);
    height: 32px; /* ROW_H in graphGeometry.ts */
    cursor: default;
  }
  .row:hover {
    background: var(--hover);
  }
  .row.selected {
    background: var(--sel);
  }
  .graph {
    overflow: hidden;
  }
  .msg {
    display: flex;
    align-items: center;
    gap: 8px;
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
  .row.selected .age {
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
  /* A fixed right-hand column, so ages line up down the list. */
  .age {
    flex: none;
    width: 30px;
    color: var(--muted);
    font-size: var(--fs-xs);
    font-variant-numeric: tabular-nums;
    text-align: right;
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
    margin: 11px 0 0 11px; /* centred on lane 0: laneX(0) = 16, ROW_H / 2 = 16 */
    border-radius: 50%;
    background: rgb(255 255 255 / 0.06);
  }
</style>
