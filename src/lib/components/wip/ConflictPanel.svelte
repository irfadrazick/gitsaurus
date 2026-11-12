<script lang="ts">
  import * as api from '../../api';
  import { runOp } from '../../state/ops';
  import { repo } from '../../state/repo.svelte';
  import { settings } from '../../state/settings.svelte';
  import type { FileStatus, Side } from '../../types';
  import { splitStatus } from '../../util/status';
  import { toWipFile } from '../../util/wip';
  import FileList from '../details/FileList.svelte';

  const split = $derived(splitStatus(repo.state.files));
  const op = $derived(repo.state.operation);
  const resolved = $derived(split.staged.map((f) => toWipFile(f, 'index')));
  const selectedPath = $derived(repo.centre.kind === 'diff' ? repo.centre.path : null);

  const base = (p: string) => p.slice(p.lastIndexOf('/') + 1);

  function openFile(f: { path: string; origPath: string | null }, kind: 'workingFile' | 'staged') {
    repo.centre = { kind: 'diff', path: f.path, origPath: f.origPath, source: { kind }, view: 'diff' };
  }
  const use = (f: FileStatus, side: Side, label: string) =>
    runOp(`Resolve ${base(f.path)} (${label})`, () => api.resolveConflict(repo.path!, f.path, side));
  const markResolved = (f: FileStatus) => runOp(`Mark ${base(f.path)} resolved`, () => api.markResolved(repo.path!, f.path));
  const editor = (f: FileStatus) =>
    runOp(`Open ${base(f.path)} in editor`, async () => {
      await api.openInEditor(repo.path!, f.path, settings.editorCommand);
      return true as const;
    });
  const unstage = (path: string, origPath: string | null) =>
    runOp(`Unstage ${base(path)}`, () => api.unstage(repo.path!, origPath ? [path, origPath] : [path]));
</script>

<div class="panel">
  <header><h3>Conflicted files ({split.conflicted.length})</h3></header>
  {#if split.conflicted.length === 0}
    <p class="empty muted small">All conflicts resolved. Continue from the banner above.</p>
  {/if}
  {#each split.conflicted as f (f.path)}
    <div class="item" class:sel={selectedPath === f.path}>
      <button class="open" type="button" title={f.path} onclick={() => openFile(f, 'workingFile')}>
        <span class="badge">U</span>
        <span class="dir">{f.path.slice(0, f.path.lastIndexOf('/') + 1)}</span><span class="name">{base(f.path)}</span>
      </button>
      <div class="btns">
        {#if op}
          <button type="button" onclick={() => use(f, 'ours', op.oursLabel)}>Use "{op.oursLabel}"</button>
          <button type="button" onclick={() => use(f, 'theirs', op.theirsLabel)}>Use "{op.theirsLabel}"</button>
        {/if}
        <button type="button" onclick={() => editor(f)}>Open in editor</button>
        <button type="button" onclick={() => markResolved(f)}>Mark resolved</button>
      </div>
    </div>
  {/each}
  {#if resolved.length}
    <header><h3>Resolved and staged ({resolved.length})</h3></header>
    <FileList bare files={resolved} onOpen={(f) => openFile(f, 'staged')} selected={selectedPath}>
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
  .item {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 4px 8px 8px;
    border-bottom: 1px solid var(--line);
  }
  .item.sel {
    background: color-mix(in srgb, var(--accent) 22%, transparent);
  }
  .open {
    display: flex;
    align-items: center;
    gap: 6px;
    min-height: 26px;
    padding: 0;
    border: 0;
    background: transparent;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    font-size: 12px;
  }
  .dir {
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--muted);
  }
  .badge {
    flex: none;
    width: 16px;
    height: 16px;
    border-radius: 4px;
    text-align: center;
    font: 700 10px/16px var(--mono);
    background: var(--del);
    color: #fff;
  }
  .btns {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .btns button {
    padding: 1px 8px;
    font-size: var(--font-min);
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .act {
    width: 22px;
    height: 20px;
    padding: 0;
    line-height: 1;
  }
</style>
