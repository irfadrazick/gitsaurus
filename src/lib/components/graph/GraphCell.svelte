<script lang="ts">
  import { avatarFailed, avatarFor } from '../../state/avatars.svelte';
  import type { GraphCommit } from '../../types';
  import { initials } from '../../util/avatar';
  import {
    LINE_W, MERGE_R, NODE_R, ROW_H, cellWidth, laneColor, laneX, seedSegment, seedStyle, segmentPath, tint,
    type SegmentStyle,
  } from '../../util/graphGeometry';

  let {
    commit,
    selected = false,
    hasRefs = false,
    index = 0,
    headIndex = null,
    hasWip = false,
    wip = false,
    wipLinked = false,
  }: {
    /** The row to draw; omitted for the WIP row. */
    commit?: GraphCommit;
    selected?: boolean;
    /** Draw the 1 px connector from the left edge (the BRANCH/TAG pill) to the node. */
    hasRefs?: boolean;
    /** Graph index of this row and of HEAD: identify the seed segments (Ruling R8). */
    index?: number;
    headIndex?: number | null;
    /** A WIP row sits above row 0: seed segments draw dashed (the WIP -> HEAD connector), else hidden. */
    hasWip?: boolean;
    /** Draw the WIP node (dashed, lane 0) instead of a commit. */
    wip?: boolean;
    /** WIP: draw the dashed link down toward HEAD. */
    wipLinked?: boolean;
  } = $props();

  const uid = $props.id();
  const cy = ROW_H / 2;
  const lane = $derived(commit?.lane ?? 0);
  const color = $derived(commit?.color ?? 0);
  const nx = $derived(laneX(lane));
  const stroke = $derived(laneColor(color));
  const w = $derived(cellWidth(commit?.width ?? 1));
  const merge = $derived((commit?.parents.length ?? 0) > 1);
  const url = $derived(commit && !merge ? avatarFor(commit.authorEmail) : null);
  // Same colour as the tinted row behind the node, so hollow nodes read as hollow.
  const rowTint = $derived(tint(color, selected ? 0.22 : 0.12));
  const DASH = '3 3';
  const styleOf = (kind: 'into' | 'pass', l: number): SegmentStyle =>
    seedStyle(seedSegment(index, headIndex, kind, l), hasWip);
</script>

<svg width={w} height={ROW_H} viewBox="0 0 {w} {ROW_H}" aria-hidden="true">
  {#if wip}
    {#if wipLinked}
      <path d="M {nx} {cy} L {nx} {ROW_H}" stroke={stroke} stroke-width={LINE_W} stroke-dasharray={DASH} fill="none" />
    {/if}
    <circle cx={nx} cy={cy} r={NODE_R} style="fill: var(--bg)" />
    <circle cx={nx} cy={cy} r={NODE_R} fill={rowTint} stroke={stroke} stroke-width={LINE_W} stroke-dasharray="4 3" />
  {:else if commit}
    <g fill="none" stroke-width={LINE_W} stroke-linecap="round">
      {#each commit.pass as l (l.lane)}
        {@const st = styleOf('pass', l.lane)}
        {#if st !== 'hidden'}
          <path d={segmentPath('pass', l.lane, lane)} stroke={laneColor(l.color)} stroke-dasharray={st === 'dashed' ? DASH : undefined} />
        {/if}
      {/each}
      {#each commit.into as l (l.lane)}
        {@const st = styleOf('into', l.lane)}
        {#if st !== 'hidden'}
          <path d={segmentPath('into', l.lane, lane)} stroke={laneColor(l.color)} stroke-dasharray={st === 'dashed' ? DASH : undefined} />
        {/if}
      {/each}
      {#each commit.out as l (l.lane)}
        <path d={segmentPath('out', l.lane, lane)} stroke={laneColor(l.color)} />
      {/each}
    </g>
    {#if hasRefs}
      <line x1="0" y1={cy + 0.5} x2={nx} y2={cy + 0.5} stroke={stroke} stroke-width="1" />
    {/if}
    {#if selected && !merge}
      <circle cx={nx} cy={cy} r={NODE_R + 3.5} fill="none" stroke={stroke} stroke-opacity="0.35" stroke-width="2" />
    {/if}
    {#if merge}
      <circle cx={nx} cy={cy} r={MERGE_R} style="fill: var(--bg)" />
      <circle cx={nx} cy={cy} r={MERGE_R} fill={rowTint} stroke={stroke} stroke-width={LINE_W} />
    {:else}
      <defs>
        <clipPath id="node-{uid}">
          <circle cx={nx} cy={cy} r={NODE_R} />
        </clipPath>
      </defs>
      <circle cx={nx} cy={cy} r={NODE_R} fill={stroke} />
      <text
        x={nx}
        y={cy}
        text-anchor="middle"
        dominant-baseline="central"
        font-size="11"
        font-weight="700"
        letter-spacing="-0.3"
        fill="#11151b">{initials(commit.authorName)}</text
      >
      {#if url}
        <image
          href={url}
          x={nx - NODE_R}
          y={cy - NODE_R}
          width={NODE_R * 2}
          height={NODE_R * 2}
          preserveAspectRatio="xMidYMid slice"
          clip-path="url(#node-{uid})"
          onerror={() => avatarFailed(commit.authorEmail)}
        />
      {/if}
      <circle cx={nx} cy={cy} r={NODE_R} fill="none" stroke={stroke} stroke-width={LINE_W} />
    {/if}
  {/if}
</svg>

<style>
  svg {
    display: block;
  }
</style>
