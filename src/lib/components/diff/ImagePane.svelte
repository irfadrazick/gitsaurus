<script lang="ts">
  import type { ImageVersion } from '../../types';
  import { formatBytes } from '../../util/image';

  let {
    side,
    label,
    version,
    mime,
    alt,
  }: { side: 'old' | 'new'; label: string; version: ImageVersion; mime: string; alt: string } = $props();

  let width = $state(0);
  let height = $state(0);
</script>

<figure>
  <figcaption>
    {#if label}<span class="label {side}">{label}</span>{/if}
    <span class="meta">
      {#if version.data && width}<span>{width} × {height}</span><span aria-hidden="true">·</span>{/if}
      <span>{formatBytes(version.size)}</span>
    </span>
  </figcaption>
  {#if version.data}
    <div class="frame">
      <img src="data:{mime};base64,{version.data}" {alt} bind:naturalWidth={width} bind:naturalHeight={height} />
    </div>
  {:else}
    <div class="note">Too large to show ({formatBytes(version.size)})</div>
  {/if}
</figure>

<style>
  figure {
    display: flex;
    min-width: 0;
    margin: 0;
    flex-direction: column;
    gap: 8px;
  }
  figcaption {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--fs-sm);
  }
  .label {
    padding: 0 7px;
    border-radius: var(--radius-xs);
    font-size: var(--fs-xs);
    font-weight: 600;
    line-height: 18px;
  }
  .label.old {
    background: color-mix(in srgb, var(--del) 18%, transparent);
    color: var(--danger-text);
  }
  .label.new {
    background: color-mix(in srgb, var(--add) 24%, transparent);
    color: var(--add);
  }
  .meta {
    display: inline-flex;
    gap: 6px;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  /* A checkerboard behind the image shows where it is transparent. */
  .frame {
    display: grid;
    place-items: center;
    min-height: 64px;
    padding: 12px;
    border: 1px solid var(--line);
    background-color: #262a31;
    background-image: conic-gradient(#2e333b 25%, transparent 0 50%, #2e333b 0 75%, transparent 0);
    background-size: 16px 16px;
  }
  img {
    display: block;
    max-width: 100%;
    height: auto;
  }
  .note {
    padding: 16px 0;
    color: var(--muted);
  }
</style>
