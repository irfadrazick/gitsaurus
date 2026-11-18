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
  import { pillTitle, pillsShown } from '../../util/refs';

  // Pills carry `data-pill={k}` (index into `pills`): GraphView resolves right-clicks by delegation.
  let { pills, color, room = 0 }: { pills: Pill[]; color: string; room?: number } = $props();

  /** Shown inline before the commit message: two when both fit in full, else the first; the rest collapse into "+N". */
  const count = $derived(pillsShown(pills.map(pillWidth), moreWidth, room));
  const shown = $derived(pills.slice(0, count));
  const rest = $derived(pills.length - shown.length);
</script>

<span class="pills" style:--c={color}>
  {#each shown as p, k (p.name + k)}
    <span class="pill" class:head={p.isHead} data-pill={k} title={pillTitle([p])}>
      {#if p.isHead}
        <svg class="ic" viewBox="0 0 16 16" aria-hidden="true"><path d="M3.5 8.5l3 3 6-7" /></svg>
      {:else if p.tag}
        <svg class="ic" viewBox="0 0 16 16" aria-hidden="true">
          <path d="M2.5 2.5h5l6 6-5 5-6-6z" /><circle cx="5.5" cy="5.5" r="1" />
        </svg>
      {/if}
      <span class="name">{p.name}</span>
      {#if p.local}
        <svg class="ic" viewBox="0 0 16 16" aria-hidden="true"><path d="M3.5 4h9v6.5h-9zM1.5 12.5h13" /></svg>
      {/if}
      {#if p.remote}
        <svg class="ic cloud" viewBox="0 0 16 16" aria-hidden="true">
          <path d="M4.6 12.5h6.9a2.8 2.8 0 0 0 .3-5.6 4.2 4.2 0 0 0-8-.9 2.9 2.9 0 0 0 .8 6.5z" />
        </svg>
      {/if}
    </span>
  {/each}
  {#if rest > 0}
    <span class="more" title={pillTitle(pills)}>+{rest}</span>
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
  .pill,
  .more {
    display: inline-flex;
    flex: none;
    align-items: center;
    gap: 4px;
    height: 20px;
    padding: 0 7px;
    border: 1px solid var(--c);
    border-radius: var(--radius-xs);
    background: color-mix(in srgb, var(--c) 22%, var(--bg));
    color: #e6edf3;
    font-size: var(--fs-xs);
    font-weight: 400;
    line-height: 1;
    white-space: nowrap;
  }
  .pill {
    flex: 0 1 auto;
    min-width: 0;
    max-width: 200px;
  }
  .pill.head {
    background: var(--c);
    color: #11151b;
  }
  .name {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    /* A line box taller than 1em, or overflow: hidden clips descenders (g, p, y). */
    line-height: 16px;
  }
  .more {
    padding: 0 5px;
    font-variant-numeric: tabular-nums;
  }
  .ic {
    flex: none;
    width: 12px;
    height: 12px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.6;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .cloud {
    opacity: 0.85;
  }
</style>
