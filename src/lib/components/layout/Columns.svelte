<script lang="ts">
  import type { Snippet } from 'svelte';
  import { session, type Widths } from '../../state/session.svelte';

  let { repos, refs, centre, details }: { repos: Snippet; refs: Snippet; centre: Snippet; details: Snippet } = $props();

  const COLLAPSED = 44;
  const MIN: Widths = { repos: 140, refs: 160, details: 260 };
  const MAX: Widths = { repos: 420, refs: 480, details: 720 };

  const reposWidth = $derived(session.reposCollapsed ? COLLAPSED : session.widths.repos);

  function startDrag(e: PointerEvent, key: keyof Widths, dir: 1 | -1) {
    e.preventDefault();
    const startX = e.clientX;
    const startW = session.widths[key];
    const target = e.currentTarget as HTMLElement;
    target.setPointerCapture(e.pointerId);
    const move = (ev: PointerEvent) => {
      const w = startW + dir * (ev.clientX - startX);
      session.widths[key] = Math.round(Math.min(MAX[key], Math.max(MIN[key], w)));
    };
    const up = () => {
      target.removeEventListener('pointermove', move);
      target.removeEventListener('pointerup', up);
      target.removeEventListener('pointercancel', up);
    };
    target.addEventListener('pointermove', move);
    target.addEventListener('pointerup', up);
    target.addEventListener('pointercancel', up);
  }
</script>

<div class="columns" style:grid-template-columns="{reposWidth}px 5px {session.widths.refs}px 5px minmax(0, 1fr) 5px {session.widths.details}px">
  <section class="col repos" class:collapsed={session.reposCollapsed}>{@render repos()}</section>
  {#if session.reposCollapsed}
    <div class="handle inert" aria-hidden="true"></div>
  {:else}
    <div class="handle" role="separator" aria-orientation="vertical" aria-label="Resize repositories" onpointerdown={(e) => startDrag(e, 'repos', 1)}></div>
  {/if}
  <section class="col">{@render refs()}</section>
  <div class="handle" role="separator" aria-orientation="vertical" aria-label="Resize refs" onpointerdown={(e) => startDrag(e, 'refs', 1)}></div>
  <section class="col centre">{@render centre()}</section>
  <div class="handle" role="separator" aria-orientation="vertical" aria-label="Resize details" onpointerdown={(e) => startDrag(e, 'details', -1)}></div>
  <section class="col">{@render details()}</section>
</div>

<style>
  .columns {
    display: grid;
    height: 100%;
    min-height: 0;
  }
  .col {
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    background: var(--panel);
  }
  .col.centre {
    background: var(--bg);
  }
  .col.collapsed {
    overflow: hidden;
  }
  .handle {
    width: 5px;
    cursor: col-resize;
    background: var(--line);
    touch-action: none;
  }
  .handle:hover {
    background: var(--accent);
  }
  .handle.inert {
    cursor: default;
    pointer-events: none;
  }
</style>
