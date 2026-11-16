<script lang="ts">
  import * as api from '../../api';
  import { repo } from '../../state/repo.svelte';
  import type { DiffSource, ImageVersion, ImageVersions } from '../../types';
  import { isLiveSource } from '../../util/liveDiff';
  import ImagePane from './ImagePane.svelte';

  let {
    path,
    origPath,
    source,
    mime,
    view,
    layout,
  }: {
    path: string;
    origPath: string | null;
    source: DiffSource;
    mime: string;
    view: 'diff' | 'file';
    layout: 'split' | 'unified';
  } = $props();

  let versions = $state.raw<ImageVersions | null>(null);
  let error = $state<string | null>(null);
  let token = 0;

  $effect(() => {
    const repoPath = repo.path;
    if (!repoPath) return;
    // Working-state sources re-read the image whenever the repo state refreshes.
    if (isLiveSource(source)) void repo.stateVersion;
    const t = ++token;
    error = null;
    api.imageVersions(repoPath, path, origPath, source).then(
      (v) => {
        if (t === token) versions = v;
      },
      (e) => {
        if (t !== token) return;
        versions = null;
        error = (e as { stderr?: string; message?: string })?.stderr || (e as { message?: string })?.message || String(e);
      },
    );
  });

  type Pane = { key: 'old' | 'new'; label: string; version: ImageVersion };
  const panes = $derived.by((): Pane[] => {
    if (!versions) return [];
    const { old, new: cur } = versions;
    if (view === 'file') {
      const shown = cur ?? old;
      return shown ? [{ key: cur ? 'new' : 'old', label: '', version: shown }] : [];
    }
    if (old && cur) {
      return [
        { key: 'old', label: 'Before', version: old },
        { key: 'new', label: 'After', version: cur },
      ];
    }
    if (cur) return [{ key: 'new', label: 'Added', version: cur }];
    if (old) return [{ key: 'old', label: 'Deleted', version: old }];
    return [];
  });

</script>

{#if error}
  <div class="note err">{error}</div>
{:else if !versions}
  <div class="note">Loading…</div>
{:else if panes.length === 0}
  <div class="note">No image to show</div>
{:else}
  <div class="panes" class:stacked={layout === 'unified' || panes.length === 1}>
    {#each panes as p (p.key)}
      <ImagePane side={p.key} label={p.label} version={p.version} {mime} alt={p.label ? `${p.label}: ${path}` : path} />
    {/each}
  </div>
{/if}

<style>
  .panes {
    display: grid;
    grid-auto-columns: minmax(0, 1fr);
    grid-auto-flow: column;
    gap: 16px;
    padding: 16px;
  }
  .panes.stacked {
    grid-auto-flow: row;
  }
  .note {
    padding: 16px;
    color: var(--muted);
  }
  .note.err {
    color: var(--danger-text);
    white-space: pre-wrap;
  }
</style>
