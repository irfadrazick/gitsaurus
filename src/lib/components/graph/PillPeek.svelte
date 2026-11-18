<script lang="ts">
  import type { HTMLAttributes } from 'svelte/elements';
  import type { Pill } from '../../types';
  import RefPill from './RefPill.svelte';

  // Hover overlay for a graph row's ref pills, fixed to the window so it can spill past the list.
  // "expand": a truncated pill shown in full, drawn exactly over the original. Display only: pointer
  //   events pass through to the pill underneath (its right-click menu still works).
  // "list": every pill of a row that has a "+N" chip, stacked over the row's pills (`anchor`), the
  //   first one in its place, growing downwards (upwards when there's no room). No panel: the pills
  //   float on their own shadows. The
  //   pointer can move into it; its pills carry `data-pill={k}` (index into `pills`) for GraphView's
  //   handlers, passed in with the rest of the element attributes.
  let {
    pills, color, mode, anchor, ...rest
  }: { pills: Pill[]; color: string; mode: 'expand' | 'list'; anchor: DOMRect } & HTMLAttributes<HTMLDivElement> = $props();

  const EDGE = 8;
  let w = $state(0);
  let h = $state(0);

  const left = $derived(Math.max(EDGE, Math.min(anchor.left, window.innerWidth - w - EDGE)));
  const top = $derived(
    anchor.top + h <= window.innerHeight - EDGE ? anchor.top : Math.max(EDGE, anchor.bottom - h),
  );
</script>

<div
  {...rest}
  class="peek"
  class:list={mode === 'list'}
  data-peek=""
  style:--c={color}
  style:left="{left}px"
  style:top="{top}px"
  style:visibility={w ? 'visible' : 'hidden'}
  bind:offsetWidth={w}
  bind:offsetHeight={h}
>
  {#each pills as p, k (p.name + k)}
    <RefPill pill={p} k={k} full />
  {/each}
</div>

<style>
  .peek {
    position: fixed;
    z-index: 20;
    display: flex;
    pointer-events: none;
  }
  .peek :global(.pill) {
    box-shadow: 0 2px 8px rgb(0 0 0 / 0.5);
  }
  .list {
    flex-direction: column;
    align-items: flex-start;
    gap: 3px;
    pointer-events: auto;
  }
  .list :global(.pill:hover) {
    filter: brightness(1.2);
  }
</style>
