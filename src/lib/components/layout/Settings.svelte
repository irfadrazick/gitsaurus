<script lang="ts">
  import { settings } from '../../state/settings.svelte';
  import { ui } from '../../state/ui.svelte';
  import { SHORTCUTS } from '../../util/shortcuts';

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

<div class="dialog-backdrop settings" role="presentation" onmousedown={(e) => e.target === e.currentTarget && close()}>
  <div class="dialog" role="dialog" aria-modal="true" aria-label="Settings">
    <h2>Settings</h2>
    <div class="field">
      <label for="fetch-min">Background fetch</label>
      <div class="inline">
        <input id="fetch-min" type="number" min="0" step="1" bind:value={minutes} onchange={saveMinutes} />
        <span class="unit">minutes</span>
      </div>
      <p class="hint">How often to fetch from remotes in the background. 0 turns it off.</p>
    </div>
    <div class="field">
      <label class="check">
        <input type="checkbox" bind:checked={settings.avatarsEnabled} />
        <span>Show author photos</span>
      </label>
      <p class="hint indent">Loads pictures from Gravatar and GitHub. Turn off to make no requests for them.</p>
    </div>
    <div class="field">
      <label for="editor-cmd">Editor command</label>
      <input id="editor-cmd" type="text" placeholder="code -w" spellcheck="false" autocomplete="off" bind:value={settings.editorCommand} />
      <p class="hint">Used by "Open in editor". Leave empty to use your default text editor.</p>
    </div>
    <div class="field">
      <h3>Keyboard shortcuts</h3>
      <dl class="keys">
        {#each SHORTCUTS as s (s.keys)}
          <dt><kbd>{s.keys}</kbd></dt>
          <dd>{s.label}</dd>
        {/each}
      </dl>
    </div>
    <div class="actions"><button onclick={close}>Done</button></div>
  </div>
</div>

<style>
  .settings {
    z-index: 90;
  }
  .dialog {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-height: calc(100vh - 48px);
    overflow-y: auto;
  }
  h3 {
    margin: 0;
    font-size: var(--fs-sm);
    font-weight: 600;
  }
  .keys {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 5px 12px;
    margin: 2px 0 0;
    font-size: var(--fs-sm);
  }
  .keys dt {
    justify-self: end;
  }
  .keys dd {
    margin: 0;
    color: color-mix(in srgb, var(--text) 85%, var(--muted));
  }
  kbd {
    display: inline-block;
    min-width: 22px;
    padding: 0 6px;
    border: 1px solid var(--line);
    border-bottom-width: 2px;
    border-radius: var(--radius-sm);
    background: var(--bg);
    color: var(--text);
    font: inherit;
    font-size: var(--fs-xs);
    line-height: 18px;
    text-align: center;
    white-space: nowrap;
  }
  h2 {
    margin-bottom: 0;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .field > label:not(.check) {
    font-size: var(--fs-sm);
    font-weight: 600;
  }
  .inline {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .inline input {
    width: 80px;
  }
  .unit {
    color: var(--muted);
    font-size: var(--fs-sm);
  }
  .hint {
    margin: 0;
    color: var(--muted);
    font-size: var(--fs-xs);
    line-height: 1.45;
  }
  .hint.indent {
    padding-left: 22px;
  }
  .check {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--fs-sm);
    font-weight: 600;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
  }
  .actions button {
    min-width: 72px;
  }
</style>
