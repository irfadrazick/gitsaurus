<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import * as api from '../../api';
  import { repo } from '../../state/repo.svelte';
  import { ui } from '../../state/ui.svelte';
  import { stepMatch } from '../../util/graphGeometry';

  // Mounted while ui.searchOpen (Ruling R3). Esc closes it. Matches are graph indices from searchCommits.
  let {
    matches = $bindable([]),
    pos = $bindable(-1),
    onjump,
  }: {
    matches?: number[];
    /** Position of the current match within `matches`, or -1. */
    pos?: number;
    /** Called with a graph index to select and scroll to. */
    onjump: (index: number) => void;
  } = $props();

  const DEBOUNCE_MS = 150;

  let query = $state('');
  let searched = $state(false);
  let input: HTMLInputElement | undefined = $state();
  let seq = 0;
  let lastQuery = '';

  onMount(() => focusInput());

  function focusInput() {
    input?.focus();
    input?.select();
  }

  // Debounced search; re-runs (without jumping) when the graph reloads, since indices may shift.
  $effect(() => {
    const q = query.trim();
    const path = repo.path;
    void repo.graphVersion;
    const my = ++seq;
    if (!q || !path) {
      untrack(() => {
        matches = [];
        pos = -1;
        searched = false;
      });
      lastQuery = '';
      return;
    }
    const timer = setTimeout(async () => {
      let res: number[];
      try {
        res = await api.searchCommits(path, q);
      } catch {
        res = [];
      }
      if (my !== seq) return;
      const sameQuery = q === lastQuery;
      lastQuery = q;
      matches = res;
      searched = true;
      if (sameQuery && pos >= 0) {
        pos = Math.min(pos, res.length - 1);
      } else {
        pos = res.length ? 0 : -1;
        if (res.length) onjump(res[0]);
      }
    }, DEBOUNCE_MS);
    return () => clearTimeout(timer);
  });

  function step(dir: 1 | -1) {
    if (!matches.length) return;
    pos = stepMatch(matches.length, pos, dir);
    onjump(matches[pos]);
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      step(e.shiftKey ? -1 : 1);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      ui.searchOpen = false;
    }
  }

  // ⌘F while already open: refocus the field (the global handler only sets searchOpen).
  function onWindowKeydown(e: KeyboardEvent) {
    if (e.metaKey && !e.ctrlKey && !e.altKey && e.key.toLowerCase() === 'f') focusInput();
  }

  const counter = $derived(searched ? `${matches.length ? pos + 1 : 0} / ${matches.length}` : '');
</script>

<svelte:window onkeydown={onWindowKeydown} />

<div class="search" role="search">
  <svg class="glass" viewBox="0 0 16 16" aria-hidden="true"><circle cx="7" cy="7" r="4.5" /><path d="M10.5 10.5l3 3" /></svg>
  <input
    bind:this={input}
    bind:value={query}
    type="text"
    placeholder="Search message, author or SHA"
    aria-label="Search commits"
    spellcheck="false"
    autocomplete="off"
    onkeydown={onKeydown}
  />
  <span class="count" class:none={searched && !matches.length} aria-live="polite">{counter}</span>
  <button class="icon" title="Previous match (⇧↩)" aria-label="Previous match" disabled={!matches.length} onclick={() => step(-1)}>
    <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 10l4-4 4 4" /></svg>
  </button>
  <button class="icon" title="Next match (↩)" aria-label="Next match" disabled={!matches.length} onclick={() => step(1)}>
    <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 6l4 4 4-4" /></svg>
  </button>
  <button class="icon" title="Close (Esc)" aria-label="Close search" onclick={() => (ui.searchOpen = false)}>
    <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4.5 4.5l7 7M11.5 4.5l-7 7" /></svg>
  </button>
</div>

<style>
  .search {
    display: flex;
    align-items: center;
    gap: 2px;
    width: 340px;
    height: 34px;
    padding: 0 4px 0 10px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
  }
  .glass {
    flex: none;
    width: 14px;
    height: 14px;
    margin-right: 4px;
    fill: none;
    stroke: var(--muted);
    stroke-width: 1.6;
    stroke-linecap: round;
  }
  input {
    flex: 1;
    min-width: 0;
    height: 26px;
    padding: 0 4px;
    border: 0;
    background: transparent;
    font-size: 13px;
  }
  input:focus {
    outline: none;
  }
  .count {
    flex: none;
    min-width: 44px;
    padding: 0 6px;
    color: var(--muted);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    text-align: right;
  }
  .count.none {
    color: var(--del);
  }
  .icon {
    display: inline-flex;
    flex: none;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: 0;
    border-radius: 5px;
    background: transparent;
    color: var(--muted);
  }
  .icon:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.07);
    color: var(--text);
  }
  .icon:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .icon svg {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.6;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
</style>
