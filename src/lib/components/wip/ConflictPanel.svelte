<script lang="ts">
  import * as api from '../../api';
  import { runOp } from '../../state/ops';
  import { repo } from '../../state/repo.svelte';
  import { splitStatus } from '../../util/status';
  import { conflictSides, toWipFile } from '../../util/wip';
  import FileList from '../details/FileList.svelte';
  import ConflictFile from './ConflictFile.svelte';

  const split = $derived(splitStatus(repo.state.files));
  const sides = $derived(conflictSides(repo.state.operation));
  const resolved = $derived(split.staged.map((f) => toWipFile(f, 'index')));
  const selectedPath = $derived(repo.centre.kind === 'diff' ? repo.centre.path : null);

  const base = (p: string) => p.slice(p.lastIndexOf('/') + 1);

  function openStaged(f: { path: string; origPath: string | null }) {
    repo.centre = { kind: 'diff', path: f.path, origPath: f.origPath, source: { kind: 'staged' }, view: 'diff' };
  }
  const unstage = (path: string, origPath: string | null) =>
    runOp(`Unstage ${base(path)}`, () => api.unstage(repo.path!, origPath ? [path, origPath] : [path]));
</script>

<div class="panel">
  <header><h3>Conflicted files ({split.conflicted.length})</h3></header>
  {#if split.conflicted.length === 0}
    <p class="empty muted small">All conflicts resolved. Continue from the banner above.</p>
  {/if}
  {#each split.conflicted as f (f.path)}
    <ConflictFile file={f} {...sides} />
  {/each}
  {#if resolved.length}
    <header><h3>Resolved and staged ({resolved.length})</h3></header>
    <FileList bare files={resolved} onOpen={openStaged} selected={selectedPath}>
      {#snippet actions(f)}
        <button class="act" type="button" title="Unstage" aria-label="Unstage {f.path}" onclick={() => unstage(f.path, f.origPath)}>−</button>
      {/snippet}
    </FileList>
  {/if}
</div>

<style>
  .panel {
    height: 100%;
    overflow-y: auto;
  }
  header {
    position: sticky;
    top: 0;
    z-index: 1;
    padding: 4px 8px;
    background: var(--panel);
    border-bottom: 1px solid var(--line);
  }
  h3 {
    margin: 0;
    font-size: 12px;
    font-weight: 600;
  }
  .empty {
    margin: 0;
    padding: 8px 12px;
  }
  .act {
    width: 22px;
    height: 20px;
    padding: 0;
    line-height: 1;
  }
</style>
