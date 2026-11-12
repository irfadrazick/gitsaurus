<script lang="ts">
  import * as api from '../../api';
  import { confirm } from '../../state/modal.svelte';
  import { openMenu } from '../../state/contextMenu.svelte';
  import { runOp } from '../../state/ops';
  import { repo } from '../../state/repo.svelte';
  import type { ChangedFile, DiffSource } from '../../types';
  import { splitStatus } from '../../util/status';
  import { toWipFile } from '../../util/wip';
  import FileList from '../details/FileList.svelte';
  import CommitBox from './CommitBox.svelte';

  const split = $derived(splitStatus(repo.state.files));
  const unstaged = $derived(split.unstaged.map((f) => toWipFile(f, 'worktree')));
  const staged = $derived(split.staged.map((f) => toWipFile(f, 'index')));
  const untracked = $derived(new Set(split.unstaged.filter((f) => f.untracked).map((f) => f.path)));
  const selectedPath = $derived(repo.centre.kind === 'diff' ? repo.centre.path : null);

  function open(file: ChangedFile, source: DiffSource) {
    repo.centre = { kind: 'diff', path: file.path, origPath: file.origPath, source, view: 'diff' };
  }
  const openUnstaged = (f: ChangedFile) => open(f, { kind: untracked.has(f.path) ? 'untracked' : 'unstaged' });
  const openStaged = (f: ChangedFile) => open(f, { kind: 'staged' });

  const base = (p: string) => p.slice(p.lastIndexOf('/') + 1);

  const stage = (f: ChangedFile) => runOp(`Stage ${base(f.path)}`, () => api.stage(repo.path!, [f.path]));
  const unstage = (f: ChangedFile) =>
    runOp(`Unstage ${base(f.path)}`, () => api.unstage(repo.path!, f.origPath ? [f.path, f.origPath] : [f.path]));
  const stageAll = () => runOp('Stage all', () => api.stageAll(repo.path!));
  const unstageAll = () => runOp('Unstage all', () => api.unstageAll(repo.path!));

  function onContext(f: ChangedFile, e: MouseEvent) {
    openMenu(e.clientX, e.clientY, [{ id: 'discard', label: 'Discard changes', danger: true }], async () => {
      const isNew = untracked.has(f.path);
      const ok = await confirm({
        title: 'Discard changes',
        body: isNew
          ? `${f.path} is untracked and will be deleted permanently. This cannot be undone.`
          : `All unstaged changes to ${f.path} will be lost. This cannot be undone.`,
        danger: true,
        confirmLabel: 'Discard',
      });
      if (ok) await runOp(`Discard ${base(f.path)}`, () => api.discard(repo.path!, [f.path]));
    });
  }
</script>

<div class="wip">
  <div class="scroll">
    <section>
      <header>
        <h3>Unstaged files ({unstaged.length})</h3>
        <button type="button" disabled={unstaged.length === 0} onclick={stageAll}>Stage all</button>
      </header>
      {#if unstaged.length}
        <FileList bare files={unstaged} onOpen={openUnstaged} selected={selectedPath} {onContext}>
          {#snippet actions(f)}
            <button class="act" type="button" title="Stage" aria-label="Stage {f.path}" onclick={() => stage(f)}>+</button>
          {/snippet}
        </FileList>
      {:else}
        <p class="empty muted small">No unstaged changes</p>
      {/if}
    </section>
    <section>
      <header>
        <h3>Staged files ({staged.length})</h3>
        <button type="button" disabled={staged.length === 0} onclick={unstageAll}>Unstage all</button>
      </header>
      {#if staged.length}
        <FileList bare files={staged} onOpen={openStaged} selected={selectedPath}>
          {#snippet actions(f)}
            <button class="act" type="button" title="Unstage" aria-label="Unstage {f.path}" onclick={() => unstage(f)}>−</button>
          {/snippet}
        </FileList>
      {:else}
        <p class="empty muted small">Nothing staged</p>
      {/if}
    </section>
  </div>
  <CommitBox staged={staged.length} />
</div>

<style>
  .wip {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }
  header {
    position: sticky;
    top: 0;
    z-index: 1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    background: var(--panel);
    border-bottom: 1px solid var(--line);
  }
  h3 {
    margin: 0;
    font-size: 12px;
    font-weight: 600;
  }
  header button {
    padding: 1px 8px;
    font-size: var(--font-min);
  }
  .act {
    width: 22px;
    height: 20px;
    padding: 0;
    line-height: 1;
  }
  .empty {
    margin: 0;
    padding: 8px 12px;
  }
</style>
