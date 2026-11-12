<script lang="ts">
  import * as api from '../../api';
  import { confirm } from '../../state/modal.svelte';
  import { openMenu } from '../../state/contextMenu.svelte';
  import { runOp } from '../../state/ops';
  import { repo } from '../../state/repo.svelte';
  import type { ChangedFile, DiffSource } from '../../types';
  import { splitStatus } from '../../util/status';
  import { conflictSides, toWipFile } from '../../util/wip';
  import FileList from '../details/FileList.svelte';
  import CommitBox from './CommitBox.svelte';
  import ConflictFile from './ConflictFile.svelte';
  import Icon from '../Icon.svelte';

  const split = $derived(splitStatus(repo.state.files));
  // Conflicts outside an operation (e.g. a stash apply/pop); conflicted files never get a Discard action.
  const sides = $derived(conflictSides(repo.state.operation));
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

  // Keyboard: ↑/↓ move through every changed file (unstaged, then staged) and show its diff;
  // Space stages or unstages the shown file and moves on to the next one in the same list.
  type Entry = { file: ChangedFile; list: 'unstaged' | 'staged' };
  const entries = $derived<Entry[]>([
    ...unstaged.map((file) => ({ file, list: 'unstaged' as const })),
    ...staged.map((file) => ({ file, list: 'staged' as const })),
  ]);
  const currentIndex = $derived.by(() => {
    if (repo.centre.kind !== 'diff') return -1;
    const { path, source } = repo.centre;
    const list = source.kind === 'staged' ? 'staged' : 'unstaged';
    return entries.findIndex((e) => e.file.path === path && e.list === list);
  });
  const show = (e: Entry) => (e.list === 'staged' ? openStaged(e.file) : openUnstaged(e.file));
  let scrollEl = $state<HTMLDivElement>();

  function onListKey(e: KeyboardEvent) {
    const el = e.target as HTMLElement;
    if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || e.metaKey || e.ctrlKey || e.altKey) return;
    const i = currentIndex;
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      const next = entries[i < 0 ? 0 : i + (e.key === 'ArrowDown' ? 1 : -1)];
      if (next) show(next);
    } else if (e.key === ' ' && i >= 0) {
      const cur = entries[i];
      const after = entries.slice(i + 1).find((x) => x.list === cur.list) ?? entries.slice(0, i).findLast((x) => x.list === cur.list);
      void (cur.list === 'unstaged' ? stage(cur.file) : unstage(cur.file));
      if (after) show(after);
    } else return;
    e.preventDefault();
    // Keep keyboard focus on the list itself, not on whichever row button was clicked last.
    scrollEl?.focus({ preventScroll: true });
  }
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
  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <div class="scroll" bind:this={scrollEl} tabindex="-1" role="region" aria-label="Changed files" onkeydown={onListKey}>
    {#if split.conflicted.length}
      <section>
        <header><h3>Conflicted files ({split.conflicted.length})</h3></header>
        {#each split.conflicted as f (f.path)}
          <ConflictFile file={f} {...sides} />
        {/each}
      </section>
    {/if}
    <section>
      <header>
        <h3>Unstaged files ({unstaged.length})</h3>
        <button type="button" disabled={unstaged.length === 0} onclick={stageAll} title="Stage all (⌘⇧S)">Stage all</button>
      </header>
      {#if unstaged.length}
        <FileList bare files={unstaged} onOpen={openUnstaged} selected={selectedPath} {onContext}>
          {#snippet actions(f)}
            <button class="act" type="button" title="Stage (Space)" aria-label="Stage {f.path}" onclick={() => stage(f)}><Icon name="plus" size={12} stroke={1.8} /></button>
          {/snippet}
        </FileList>
      {:else}
        <p class="empty muted small">No unstaged changes</p>
      {/if}
    </section>
    <section>
      <header>
        <h3>Staged files ({staged.length})</h3>
        <button type="button" disabled={staged.length === 0} onclick={unstageAll} title="Unstage all (⌘⇧U)">Unstage all</button>
      </header>
      {#if staged.length}
        <FileList bare files={staged} onOpen={openStaged} selected={selectedPath}>
          {#snippet actions(f)}
            <button class="act" type="button" title="Unstage (Space)" aria-label="Unstage {f.path}" onclick={() => unstage(f)}><Icon name="minus" size={12} stroke={1.8} /></button>
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
    outline: none;
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
    font-size: var(--fs-sm);
    font-weight: 600;
  }
  header button {
    padding: 1px 8px;
    font-size: var(--fs-xs);
  }
  section + section {
    margin-top: 6px;
  }
  .act {
    display: grid;
    place-items: center;
    width: 22px;
    height: 20px;
    padding: 0;
  }
  .empty {
    margin: 0;
    padding: 8px 12px;
  }
</style>
