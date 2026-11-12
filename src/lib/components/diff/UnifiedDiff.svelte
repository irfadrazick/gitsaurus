<script lang="ts">
  import type { DiffLine, Hunk } from '../../types';
  import './diff.css';

  let { hunks, html }: { hunks: Hunk[]; html: ReadonlyMap<DiffLine, string> } = $props();

  const marker = (k: DiffLine['kind']) => (k === 'add' ? '+' : k === 'del' ? '-' : ' ');
</script>

<div class="diff">
  {#each hunks as h, hi (hi)}
    <div class="hunk-head" data-hunk>{h.header}</div>
    {#each h.lines as l, li (li)}
      <div class="line {l.kind}">
        <span class="gutter">{l.oldNo ?? ''}</span>
        <span class="gutter">{l.newNo ?? ''}</span>
        {#if l.kind === 'noNewline'}
          <span class="mark"></span><span class="code nonl">{l.text}</span>
        {:else}
          <span class="mark">{marker(l.kind)}</span><span class="code">{#if html.has(l)}{@html html.get(l)}{:else}{l.text}{/if}</span>
        {/if}
      </div>
    {/each}
  {/each}
</div>
