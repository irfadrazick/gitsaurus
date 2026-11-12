<script lang="ts">
  import type { Pill } from '../../types';
  import { pillTitle } from '../../util/refs';

  // Pills carry `data-pill={k}` (index into `pills`): GraphView resolves right-clicks by delegation.
  let { pills, color }: { pills: Pill[]; color: string } = $props();

  const first = $derived(pills[0]);
  const rest = $derived(pills.length - 1);
</script>

{#if first}
  <div class="pills" style:--c={color}>
    <span class="pill" class:head={first.isHead} data-pill="0" title={pillTitle([first])}>
      {#if first.isHead}
        <svg class="ic" viewBox="0 0 16 16" aria-hidden="true"><path d="M3.5 8.5l3 3 6-7" /></svg>
      {:else if first.tag}
        <svg class="ic" viewBox="0 0 16 16" aria-hidden="true">
          <path d="M2.5 2.5h5l6 6-5 5-6-6z" /><circle cx="5.5" cy="5.5" r="1" />
        </svg>
      {/if}
      <span class="name">{first.name}</span>
      {#if first.remote}
        <svg class="ic cloud" viewBox="0 0 16 16" aria-hidden="true">
          <path d="M4.6 12.5h6.9a2.8 2.8 0 0 0 .3-5.6 4.2 4.2 0 0 0-8-.9 2.9 2.9 0 0 0 .8 6.5z" />
        </svg>
      {/if}
    </span>
    {#if rest > 0}
      <span class="more" title={pillTitle(pills)}>+{rest}</span>
    {/if}
    <span class="wire"></span>
  </div>
{/if}

<style>
  .pills {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 100%;
    min-width: 0;
    padding-left: 8px;
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
    border-radius: 4px;
    background: color-mix(in srgb, var(--c) 22%, var(--bg));
    color: #e6edf3;
    font-size: 11px;
    font-weight: 600;
    line-height: 1;
    white-space: nowrap;
  }
  .pill {
    flex: 0 1 auto;
    min-width: 0;
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
  .wire {
    flex: 1 0 6px;
    align-self: flex-start;
    height: 1px;
    margin: 18px 0 0 -4px;
    background: var(--c);
  }
</style>
