<script lang="ts">
  import { toasts } from '../../state/toasts.svelte';
  import Icon from '../Icon.svelte';

  const ICON = { ok: 'checkCircle', info: 'info', error: 'alert' } as const;
</script>

<div class="toasts" role="status" aria-live="polite">
  {#each toasts.items as t (t.id)}
    <div class="toast {t.kind}">
      <span class="kind" aria-hidden="true"><Icon name={ICON[t.kind]} size={16} /></span>
      <div class="body">
        <div class="title">{t.title}</div>
        {#if t.detail}<div class="detail mono">{t.detail}</div>{/if}
        {#if t.action}
          {@const a = t.action}
          <button class="primary act" onclick={() => { toasts.dismiss(t.id); a.run(); }}>{a.label}</button>
        {/if}
      </div>
      <button class="iconbtn x" aria-label="Dismiss" onclick={() => toasts.dismiss(t.id)}><Icon name="close" size={12} /></button>
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
    gap: 10px;
    align-items: flex-start;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--radius-lg);
    padding: 10px 6px 10px 12px;
    box-shadow: var(--shadow-pop);
  }
  .kind {
    display: flex;
    padding-top: 1px;
    color: var(--add);
  }
  .toast.info .kind {
    color: var(--focus);
  }
  .toast.error .kind {
    color: var(--danger-text);
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
    font-size: var(--fs-xs);
    color: var(--muted);
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 120px;
    overflow: auto;
    user-select: text;
    -webkit-user-select: text;
  }
  .act {
    margin-top: 8px;
    padding: 3px 10px;
    font-size: var(--fs-sm);
  }
  .x {
    width: 20px;
    height: 20px;
  }
</style>
