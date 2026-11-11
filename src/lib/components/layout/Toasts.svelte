<script lang="ts">
  import { toasts } from '../../state/toasts.svelte';
</script>

<div class="toasts" role="status" aria-live="polite">
  {#each toasts.items as t (t.id)}
    <div class="toast {t.kind}">
      <div class="body">
        <div class="title">{t.title}</div>
        {#if t.detail}<div class="detail mono">{t.detail}</div>{/if}
      </div>
      <button class="x" aria-label="Dismiss" onclick={() => toasts.dismiss(t.id)}>&times;</button>
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    right: 12px;
    bottom: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 200;
    max-width: 420px;
  }
  .toast {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    background: var(--panel);
    border: 1px solid var(--line);
    border-left: 3px solid var(--add);
    border-radius: 6px;
    padding: 8px 10px;
    box-shadow: 0 4px 16px rgb(0 0 0 / 0.4);
  }
  .toast.error {
    border-left-color: var(--del);
  }
  .body {
    flex: 1;
    min-width: 0;
  }
  .title {
    font-weight: 600;
  }
  .detail {
    margin-top: 4px;
    font-size: 11px;
    color: var(--muted);
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 120px;
    overflow: auto;
    user-select: text;
  }
  .x {
    border: 0;
    background: none;
    padding: 0 4px;
    font-size: 16px;
    line-height: 1;
    color: var(--muted);
  }
</style>
