<script lang="ts">
  import * as api from '../../api';
  import { runOp } from '../../state/ops';
  import { repo } from '../../state/repo.svelte';
  import { settings } from '../../state/settings.svelte';
  import type { FileStatus, Side } from '../../types';
  import Icon from '../Icon.svelte';

  /** One conflicted file with its resolve actions. `ours`/`theirs` are the side names shown on the buttons. */
  let { file, ours, theirs }: { file: FileStatus; ours: string; theirs: string } = $props();

  const base = (p: string) => p.slice(p.lastIndexOf('/') + 1);
  const dir = (p: string) => p.slice(0, p.lastIndexOf('/') + 1);
  // Side names arrive quoted for the old "Use "main"" wording; the buttons show them bare.
  const bare = (s: string) => s.replace(/^"(.*)"$/, '$1');
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
  <div class="head">
    <button class="open" type="button" title={file.path} onclick={open}>
      <span class="badge" title="Conflicted">U</span>
      <span class="pathtext">{#if dir(file.path)}<span class="dir">{dir(file.path)}</span>{/if}<span class="name">{base(file.path)}</span></span>
    </button>
    <button class="iconbtn" type="button" title="Open in editor" aria-label="Open {base(file.path)} in editor" onclick={editor}><Icon name="pencil" /></button>
    <button class="resolve" type="button" title="Stage the file as resolved" onclick={markResolved}>Mark resolved</button>
  </div>
  <div class="take">
    <span class="label">Take file from</span>
    <button type="button" title="Replace {base(file.path)} with the {bare(ours)} version" onclick={() => use('ours', ours)}>{bare(ours)}</button>
    <button type="button" title="Replace {base(file.path)} with the {bare(theirs)} version" onclick={() => use('theirs', theirs)}>{bare(theirs)}</button>
  </div>
</div>

<style>
  .item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 4px 8px 8px;
    border-bottom: 1px solid var(--line);
  }
  .item.sel {
    background: var(--sel);
  }
  .head {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .open {
    display: flex;
    flex: 1;
    min-width: 0;
    align-items: center;
    gap: 6px;
    min-height: 28px;
    padding: 0;
    border: 0;
    background: transparent;
    text-align: left;
    white-space: nowrap;
    font-size: var(--fs-sm);
  }
  .open:hover:not(:disabled),
  .open:active:not(:disabled) {
    background: transparent;
  }
  .open:hover .name {
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  .pathtext {
    display: flex;
    min-width: 0;
    overflow: hidden;
  }
  .dir {
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--muted);
  }
  .name {
    flex: none;
  }
  .badge {
    flex: none;
    width: 16px;
    height: 16px;
    border-radius: var(--radius-sm);
    text-align: center;
    font: 700 10px/16px var(--mono);
    background: var(--danger-fill);
    color: var(--on-accent);
  }
  .resolve {
    flex: none;
    padding: 1px 8px;
    font-size: var(--fs-xs);
  }
  .take {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
    padding-left: 22px;
  }
  .label {
    flex: none;
    margin-right: 2px;
    color: var(--muted);
    font-size: var(--fs-xs);
  }
  .take button {
    min-width: 0;
    padding: 1px 8px;
    font-size: var(--fs-xs);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
