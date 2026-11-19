<script module lang="ts">
  import type { Pill } from '../../types';

  // Natural widths, from the text and the pill CSS below, so a row can decide how many pills fit
  // without laying them out first. Keep these in step with the styles.
  const PILL_CHROME = 16; // 1 px border + 7 px padding, each side
  const MORE_CHROME = 12; // 1 px border + 5 px padding, each side
  const ICON = 12;
  const INNER_GAP = 4;
  const PILL_MAX = 200;

  let ctx: CanvasRenderingContext2D | null | undefined;
  const textWidths = new Map<string, number>();

  function textWidth(text: string): number {
    let w = textWidths.get(text);
    if (w !== undefined) return w;
    if (ctx === undefined) {
      ctx = document.createElement('canvas').getContext('2d');
      if (ctx) {
        const root = getComputedStyle(document.documentElement);
        ctx.font = `400 ${root.getPropertyValue('--fs-xs').trim() || '10.5px'} ${getComputedStyle(document.body).fontFamily}`;
      }
    }
    // Without a canvas, a generous estimate errs towards showing fewer, whole pills.
    w = ctx ? Math.ceil(ctx.measureText(text).width) + 1 : text.length * 7;
    textWidths.set(text, w);
    return w;
  }

  function pillWidth(p: Pill): number {
    const icons = (p.isHead || p.tag ? 1 : 0) + (p.local ? 1 : 0) + (p.remote ? 1 : 0);
    return Math.min(PILL_MAX, PILL_CHROME + textWidth(p.name) + icons * (ICON + INNER_GAP));
  }

  const moreWidth = (n: number) => MORE_CHROME + textWidth(`+${n}`);
</script>

<script lang="ts">
  import { pillsShown } from '../../util/refs';
  import RefPill from './RefPill.svelte';

  // Pills carry `data-pill={k}` (index into `pills`), and a row with hidden pills `data-overflow`:
  // GraphView resolves right-clicks and hovers by delegation.
  let { pills, color, room = 0 }: { pills: Pill[]; color: string; room?: number } = $props();

  /** Shown inline before the commit message: two when both fit in full, else the first; the rest collapse into "+N". */
  const count = $derived(pillsShown(pills.map(pillWidth), moreWidth, room));
  const shown = $derived(pills.slice(0, count));
  const rest = $derived(pills.length - shown.length);
</script>

<span class="pills" style:--c={color} data-overflow={rest > 0 ? '' : undefined}>
  {#each shown as p, k (p.name + k)}
    <RefPill pill={p} k={k} />
  {/each}
  {#if rest > 0}
    <RefPill more={rest} />
  {/if}
</span>

<style>
  .pills {
    display: inline-flex;
    flex: 0 1 auto;
    align-items: center;
    gap: 4px;
    min-width: 0;
    max-width: 50%;
  }
</style>
