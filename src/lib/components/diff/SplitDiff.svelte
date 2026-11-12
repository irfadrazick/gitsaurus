<script lang="ts">
  import type { DiffLine, Hunk } from '../../types';
  import { toSplitRows } from '../../util/split';
  import './diff.css';

  let { hunks, html }: { hunks: Hunk[]; html: ReadonlyMap<DiffLine, string> } = $props();

  // Conflict markers (<<<<<<< ======= >>>>>>> |||||||) in a conflicted file.
  const cmark = (t: string | undefined) => !!t && /^(<{7}|={7}|>{7}|\|{7})( |$)/.test(t);
</script>

{#snippet half(l: DiffLine | undefined, side: 'old' | 'new')}
  <div class="half {l ? l.kind : 'empty'}" class:cmark={cmark(l?.text)}>
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

<!-- Two equal halves that always fit the pane; long lines wrap so both sides stay visible and aligned. -->
<div class="diff split">
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
