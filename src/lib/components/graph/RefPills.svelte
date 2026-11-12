<script lang="ts">
  import type { Pill } from '../../types';
  import { pillTitle } from '../../util/refs';

  // Pills carry `data-pill={k}` (index into `pills`): GraphView resolves right-clicks by delegation.
  let { pills, color }: { pills: Pill[]; color: string } = $props();

  /** Shown inline before the commit message; the rest collapse into "+N". */
  const MAX_SHOWN = 2;
  const shown = $derived(pills.slice(0, MAX_SHOWN));
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
    font-weight: 600;
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
