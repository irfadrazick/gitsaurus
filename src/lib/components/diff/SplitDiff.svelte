<script lang="ts">
  import type { DiffLine, Hunk } from '../../types';
  import { toSplitRows } from '../../util/split';
  import './diff.css';

  let { hunks, html }: { hunks: Hunk[]; html: ReadonlyMap<DiffLine, string> } = $props();

  // Both halves are as wide as the longest line, so rows stay aligned while the body scrolls sideways.
  const cols = $derived(Math.max(40, ...hunks.flatMap((h) => h.lines.map((l) => l.text.length))));
</script>

{#snippet half(l: DiffLine | undefined, side: 'old' | 'new')}
  <div class="half {l ? l.kind : 'empty'}">
    <span class="gutter">{(side === 'old' ? l?.oldNo : l?.newNo) ?? ''}</span>
    {#if l?.kind === 'noNewline'}
      <span class="code nonl">{l.text}</span>
    {:else if l}
      <span class="code">{#if html.has(l)}{@html html.get(l)}{:else}{l.text}{/if}</span>
    {:else}
      <span class="code"></span>
    {/if}
  </div>
{/snippet}

<div class="diff split" style:--cols="{cols}ch">
  {#each hunks as h, hi (hi)}
    <div class="hunk-head" data-hunk>{h.header}</div>
    {#each toSplitRows(h) as r, ri (ri)}
      <div class="line splitrow">
        {@render half(r.left, 'old')}
        {@render half(r.right, 'new')}
      </div>
    {/each}
  {/each}
</div>
