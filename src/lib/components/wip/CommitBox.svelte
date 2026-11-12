<script module lang="ts">
  // Per-repo drafts survive repo switches and the panel unmounting (in memory only).
  const drafts = new Map<string, { summary: string; description: string }>();
</script>

<script lang="ts">
  import { tick } from 'svelte';
  import * as api from '../../api';
  import { busy, runOp } from '../../state/ops';
  import { repo } from '../../state/repo.svelte';
  import { toasts } from '../../state/toasts.svelte';
  import { canCommit, commitMessageParts } from '../../util/wip';

  let { staged }: { staged: number } = $props();

  const path = repo.path ?? '';
  const initial = drafts.get(path);
  let summary = $state(initial?.summary ?? '');
  let description = $state(initial?.description ?? '');
  let amend = $state(false);
  let summaryEl = $state<HTMLInputElement>();

  const working = $derived(busy.has('Commit'));
  const ready = $derived(canCommit(summary, staged, amend) && !working);

  function onInput() {
    if (!amend) drafts.set(path, { summary, description });
  }

  async function toggleAmend(e: Event) {
    amend = (e.currentTarget as HTMLInputElement).checked;
    if (amend) {
      // Just ticked: the draft is already saved; replace the fields with HEAD's message.
      try {
        const msg = await api.lastCommitMessage(path);
        // Unticked while the message was loading: keep the restored draft.
        if (!amend) return;
        [summary, description] = msg;
      } catch (e) {
        amend = false;
        const err = e as { message?: string };
        toasts.push({ kind: 'error', title: 'No previous commit to amend', detail: err?.message ?? String(e) });
      }
    } else {
      const d = drafts.get(path);
      summary = d?.summary ?? '';
      description = d?.description ?? '';
    }
  }

  async function commit() {
    if (!ready) return;
    const ok = await runOp('Commit', async () => {
      const m = commitMessageParts(summary, description);
      await api.commit(path, m.summary, m.description, amend);
      return true as const;
    });
    if (!ok) return;
    drafts.delete(path);
    summary = '';
    description = '';
    amend = false;
    await tick();
    summaryEl?.focus();
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && e.metaKey) {
      e.preventDefault();
      commit();
    }
  }
</script>

<!-- ⌘↩ only fires while focus is inside this box. -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="box" role="group" aria-label="Commit" onkeydown={onKeydown}>
  <div class="summary">
    <input
      bind:this={summaryEl}
      bind:value={summary}
      oninput={onInput}
      type="text"
      placeholder="Summary"
      aria-label="Commit summary"
      spellcheck="false"
    />
    <span class="count mono" class:over={summary.length > 72} title="Summary length (72 recommended)">{summary.length}</span>
  </div>
  <textarea bind:value={description} oninput={onInput} rows="4" placeholder="Description" aria-label="Commit description"></textarea>
  <div class="foot">
    <label class="amend">
      <input type="checkbox" checked={amend} onchange={toggleAmend} /> Amend previous commit
    </label>
    <button class="primary" type="button" disabled={!ready} title="Commit (⌘↩)" onclick={commit}>
      {amend ? 'Amend commit' : 'Commit'}
    </button>
  </div>
</div>

<style>
  .box {
    display: flex;
    flex: none;
    flex-direction: column;
    gap: 6px;
    padding: 8px;
    border-top: 1px solid var(--line);
    background: var(--panel);
  }
  .summary {
    position: relative;
    display: flex;
  }
  .summary input {
    width: 100%;
    padding-right: 34px;
  }
  .count {
    position: absolute;
    top: 50%;
    right: 8px;
    transform: translateY(-50%);
    font-size: var(--font-min);
    color: var(--muted);
  }
  .count.over {
    color: var(--lane-5);
  }
  textarea {
    resize: vertical;
    min-height: 56px;
    max-height: 240px;
    font: inherit;
    font-size: 12px;
  }
  .foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .amend {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--muted);
  }
  .primary {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }
  .primary:disabled {
    opacity: 0.45;
  }
</style>
