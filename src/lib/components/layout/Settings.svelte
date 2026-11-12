<script lang="ts">
  import { settings } from '../../state/settings.svelte';
  import { ui } from '../../state/ui.svelte';

  let minutes = $state(String(settings.fetchIntervalMin));

  function saveMinutes() {
    const n = Math.floor(Number(minutes));
    settings.fetchIntervalMin = Number.isFinite(n) && n >= 0 ? n : 0;
    minutes = String(settings.fetchIntervalMin);
  }

  function close() {
    ui.settingsOpen = false;
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      close();
    }
  }
</script>

<svelte:window onkeydowncapture={onKey} />

<div class="backdrop" role="presentation" onmousedown={(e) => e.target === e.currentTarget && close()}>
  <div class="dialog" role="dialog" aria-modal="true" aria-label="Settings">
    <h2>Settings</h2>
    <label class="field">
      <span>Background fetch interval (minutes, 0 = off)</span>
      <input type="number" min="0" step="1" bind:value={minutes} onchange={saveMinutes} />
    </label>
    <label class="check">
      <input type="checkbox" bind:checked={settings.avatarsEnabled} />
      <span>Load avatars from Gravatar/GitHub</span>
    </label>
    <label class="field">
      <span>Editor command (empty = system default)</span>
      <input
        type="text"
        placeholder="code -w"
        spellcheck="false"
        autocomplete="off"
        bind:value={settings.editorCommand}
      />
    </label>
    <div class="actions"><button class="primary" onclick={close}>Done</button></div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgb(0 0 0 / 0.5);
    display: grid;
    place-items: center;
    z-index: 90;
  }
  .dialog {
    width: min(440px, calc(100vw - 32px));
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 8px;
    padding: 16px;
    box-shadow: 0 12px 40px rgb(0 0 0 / 0.5);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  h2 {
    margin: 0;
    font-size: 14px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
    color: var(--muted);
  }
  .check {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
  }
</style>
