<script lang="ts">
  import type { Pill } from '../../types';

  // One ref pill, or the "+N" chip when `more` is set. The lane colour comes from `--c` on an ancestor.
  // `full` lifts the width cap, for the hover overlay that shows a truncated name in full; `active`
  // marks the pill whose context menu is open (styled by the overlay).
  let { pill, more, k, full = false, active = false }: {
    pill?: Pill; more?: number; k?: number; full?: boolean; active?: boolean;
  } = $props();
</script>

{#if pill}
  <span class="pill" class:head={pill.isHead} class:full class:active data-pill={k}>
    {#if pill.isHead}
      <svg class="ic" viewBox="0 0 16 16" aria-hidden="true"><path d="M3.5 8.5l3 3 6-7" /></svg>
    {:else if pill.tag}
      <svg class="ic" viewBox="0 0 16 16" aria-hidden="true">
        <path d="M2.5 2.5h5l6 6-5 5-6-6z" /><circle cx="5.5" cy="5.5" r="1" />
      </svg>
    {/if}
    <span class="name">{pill.name}</span>
    {#if pill.local}
      <svg class="ic" viewBox="0 0 16 16" aria-hidden="true"><path d="M3.5 4h9v6.5h-9zM1.5 12.5h13" /></svg>
    {/if}
    {#if pill.remote}
      <svg class="ic cloud" viewBox="0 0 16 16" aria-hidden="true">
        <path d="M4.6 12.5h6.9a2.8 2.8 0 0 0 .3-5.6 4.2 4.2 0 0 0-8-.9 2.9 2.9 0 0 0 .8 6.5z" />
      </svg>
    {/if}
  </span>
{:else if more !== undefined}
  <span class="more">+{more}</span>
{/if}

<style>
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
  .pill.full {
    max-width: none;
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
