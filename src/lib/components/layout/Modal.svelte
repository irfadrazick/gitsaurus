<script lang="ts">
  import { closeModal, modalState } from '../../state/modal.svelte';
  import { shouldEnterConfirm } from '../../util/modalKeys';

  const req = $derived(modalState.current);
  let value = $state('');
  let error = $state<string | null>(null);
  let seq = 0;
  let inputEl = $state<HTMLInputElement>();
  let cancelEl = $state<HTMLButtonElement>();

  // Reset local fields whenever a new request appears.
  $effect(() => {
    const r = req;
    seq++;
    error = null;
    value = r?.type === 'prompt' ? (r.initial ?? '') : '';
    if (r?.type === 'prompt') queueMicrotask(() => inputEl?.select());
    else if (r?.type === 'confirm' && r.danger) queueMicrotask(() => cancelEl?.focus());
  });

  const options = $derived(
    req?.type === 'choose'
      ? req.options.map((o) => (typeof o === 'string' ? { value: o, label: o } : o))
      : [],
  );

  function finish(result: boolean | string | null) {
    const r = req;
    if (!r) return;
    (r.resolve as (v: typeof result) => void)(result);
    closeModal();
  }

  function cancel() {
    if (!req) return;
    finish(req.type === 'confirm' ? false : null);
  }

  // Validate after a short pause in typing, so the error doesn't flash on every keystroke.
  // Any pending error clears as soon as the user types again.
  let pause: ReturnType<typeof setTimeout> | undefined;
  function validate(v: string) {
    if (req?.type !== 'prompt' || !req.validate) return;
    const check = req.validate;
    const mine = ++seq;
    error = null;
    clearTimeout(pause);
    pause = setTimeout(async () => {
      const msg = await check(v);
      if (mine === seq) error = msg;
    }, 400);
  }

  async function submitPrompt() {
    if (req?.type !== 'prompt') return;
    clearTimeout(pause);
    if (req.validate) {
      const mine = ++seq;
      const msg = await req.validate(value);
      if (mine !== seq) return;
      error = msg;
      if (msg) return;
    }
    finish(value);
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      cancel();
    } else if (e.key === 'Enter' && req && shouldEnterConfirm(req.type, req.type === 'confirm' && !!req.danger, document.activeElement?.tagName ?? null)) {
      e.preventDefault();
      finish(true);
    }
  }
</script>

<svelte:window onkeydowncapture={req ? onKey : undefined} />

{#if req}
  <div class="dialog-backdrop" role="presentation" onmousedown={(e) => e.target === e.currentTarget && cancel()}>
    <div class="dialog" role="dialog" aria-modal="true" aria-label={req.title}>
      <h2>{req.title}</h2>

      {#if req.type === 'confirm'}
        <p class="body">{req.body}</p>
        <div class="actions">
          <button bind:this={cancelEl} onclick={cancel}>Cancel</button>
          <button class={req.danger ? 'danger' : 'primary'} onclick={() => finish(true)}>{req.confirmLabel}</button>
        </div>
      {:else if req.type === 'prompt'}
        <form
          onsubmit={(e) => {
            e.preventDefault();
            submitPrompt();
          }}
        >
          <label class="field">
            <span class="small muted">{req.label}</span>
            <input
              bind:this={inputEl}
              bind:value
              oninput={() => validate(value)}
              spellcheck="false"
              autocomplete="off"
            />
          </label>
          {#if error}<div class="err small">{error}</div>{/if}
          <div class="actions">
            <button type="button" onclick={cancel}>Cancel</button>
            <button type="submit" class="primary" disabled={!!error}>OK</button>
          </div>
        </form>
      {:else}
        <div class="choices">
          {#each options as o (o.value)}
            <button onclick={() => finish(o.value)}>{o.label}</button>
          {/each}
        </div>
        <div class="actions"><button onclick={cancel}>Cancel</button></div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .body {
    margin: 0 0 12px;
    max-height: 50vh;
    overflow-y: auto;
    color: color-mix(in srgb, var(--text) 85%, var(--muted));
    line-height: 1.5;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    user-select: text;
    -webkit-user-select: text;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .err {
    color: var(--danger-text);
    margin-top: 6px;
  }
  .choices {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 12px;
  }
  .choices button {
    text-align: left;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }
  .actions button {
    min-width: 72px;
  }
</style>
