<script lang="ts">
  import * as api from '../../api';
  import { runOp } from '../../state/ops';
  import { repo } from '../../state/repo.svelte';
  import { settings } from '../../state/settings.svelte';
  import type { FileStatus, Side } from '../../types';

  /** One conflicted file with its resolve actions. `ours`/`theirs` are the side names shown on the buttons. */
  let { file, ours, theirs }: { file: FileStatus; ours: string; theirs: string } = $props();

  const base = (p: string) => p.slice(p.lastIndexOf('/') + 1);
  const selected = $derived(repo.centre.kind === 'diff' && repo.centre.path === file.path);

  function open() {
    repo.centre = { kind: 'diff', path: file.path, origPath: file.origPath, source: { kind: 'workingFile' }, view: 'diff' };
  }
  const use = (side: Side, label: string) =>
    runOp(`Resolve ${base(file.path)} (${label})`, () => api.resolveConflict(repo.path!, file.path, side));
  const markResolved = () => runOp(`Mark ${base(file.path)} resolved`, () => api.markResolved(repo.path!, file.path));
  const editor = () =>
    runOp(`Open ${base(file.path)} in editor`, async () => {
      await api.openInEditor(repo.path!, file.path, settings.editorCommand);
      return true as const;
    });
</script>

<div class="item" class:sel={selected}>
  <button class="open" type="button" title={file.path} onclick={open}>
    <span class="badge">U</span>
    <span class="dir">{file.path.slice(0, file.path.lastIndexOf('/') + 1)}</span><span class="name">{base(file.path)}</span>
  </button>
  <div class="btns">
    <button type="button" onclick={() => use('ours', ours)}>Use {ours}</button>
    <button type="button" onclick={() => use('theirs', theirs)}>Use {theirs}</button>
    <button type="button" onclick={editor}>Open in editor</button>
    <button type="button" onclick={markResolved}>Mark resolved</button>
  </div>
</div>

<style>
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
</style>
