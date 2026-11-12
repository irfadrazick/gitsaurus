<script lang="ts">
  import type { GraphCommit } from '../../types';
  import { laneColor, tint } from '../../util/graphGeometry';
  import { rowPills } from '../../util/refs';
  import Avatar from './Avatar.svelte';
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
    style:--tint={tint(0, selected ? 0.22 : 0.12)}
  >
    <div class="refs"></div>
    <div class="graph"><GraphCell wip wipLinked={wipLinked} selected={selected} /></div>
    <div class="msg">
      <span class="text"><span class="wiplabel">// WIP</span><span class="body">{wipCount} changed {wipCount === 1 ? 'file' : 'files'}</span></span>
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
    style:--tint={tint(color, selected ? 0.22 : 0.12)}
  >
    <div class="refs"><RefPills pills={pills} color={laneColor(color)} /></div>
    <div class="graph">
      <GraphCell commit={commit} selected={selected} hasRefs={pills.length > 0} index={index} headIndex={headIndex} hasWip={hasWip} />
    </div>
    <div class="msg">
      <span class="text"><span class="summary">{commit.summary}</span>{#if commit.bodyPreview}<span class="body">{commit.bodyPreview}</span>{/if}</span>
      {#if marker}<span class="marker">{marker}</span>{/if}
      <Avatar email={commit.authorEmail} name={commit.authorName} color={laneColor(color)} />
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
  .row {
    display: grid;
    grid-template-columns: var(--refs-w) var(--graph-w) minmax(0, 1fr);
    height: 36px;
    cursor: default;
  }
  .refs {
    min-width: 0;
    overflow: hidden;
  }
  .graph {
    overflow: hidden;
    background: var(--tint);
  }
  .msg {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
    padding: 0 10px 0 6px;
    background: var(--tint);
  }
  .row:hover .graph,
  .row:hover .msg {
    background-image: linear-gradient(rgba(255, 255, 255, 0.035), rgba(255, 255, 255, 0.035));
  }
  .row.selected .msg {
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04), inset 0 -1px 0 rgba(0, 0, 0, 0.25);
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
    color: #f0f3f6;
  }
  .body {
    margin-left: 8px;
    color: var(--muted);
    font-size: 12px;
  }
  .wiplabel {
    color: var(--muted);
    font-family: var(--mono);
    font-size: 12px;
    font-style: italic;
  }
  .wip .body {
    color: var(--text);
    font-size: 13px;
  }
  .marker {
    flex: none;
    padding: 0 7px;
    border-radius: 9px;
    background: rgba(255, 255, 255, 0.07);
    color: var(--muted);
    font-size: 11px;
    line-height: 18px;
    white-space: nowrap;
  }
  /* Search hits: an amber edge on the message cell; the current hit is stronger. */
  .row.match .msg {
    box-shadow: inset 3px 0 0 rgba(227, 179, 65, 0.55);
  }
  .row.match .summary {
    color: #f2d27a;
  }
  .row.current .msg {
    box-shadow: inset 3px 0 0 #e3b341, inset 0 0 0 1px rgba(227, 179, 65, 0.45);
  }
  .skeleton .graph,
  .skeleton .msg {
    background: transparent;
  }
  .dot {
    display: block;
    width: 10px;
    height: 10px;
    margin: 13px 0 0 9px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.06);
  }
  .bar {
    height: 10px;
    border-radius: 5px;
    background: linear-gradient(90deg, rgba(255, 255, 255, 0.05), rgba(255, 255, 255, 0.09), rgba(255, 255, 255, 0.05));
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
