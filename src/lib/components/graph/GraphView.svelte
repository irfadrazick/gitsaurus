<script lang="ts">
  import { onMount, tick, untrack } from 'svelte';
  import * as api from '../../api';
  import { performMenuAction, type ActionContext } from '../../state/actions';
  import { contextMenu, openMenu } from '../../state/contextMenu.svelte';
  import { PAGE_SIZE, repo } from '../../state/repo.svelte';
  import { ui } from '../../state/ui.svelte';
  import type { GraphCommit, Pill } from '../../types';
  import type { MenuKind } from '../../util/actionTable';
  import {
    GRAPH_COL_MIN, OVERSCAN, ROW_H, graphColumnWidth, keepVisibleScrollTop, laneColor, pagesFor, revealScrollTop,
    visibleRange,
  } from '../../util/graphGeometry';
  import { branchMenu, commitMenu, remoteBranchMenu, tagMenu, type MenuItem } from '../../util/menus';
  import { rowPills } from '../../util/refs';
  import { shortAge } from '../../util/time';
  import GraphRow from './GraphRow.svelte';
  import PillPeek from './PillPeek.svelte';
  import SearchBar from './SearchBar.svelte';

  // --- layout ---------------------------------------------------------------------------------
  const GRAPH_MIN = GRAPH_COL_MIN;
  const GRAPH_MAX = 900;
  /** User-resizable cap; the graph column is sized to max lanes up to this. */
  let graphCap = $state(264);
  const graphW = $derived(graphColumnWidth(repo.summary.maxLanes, graphCap));

  // --- virtual list ---------------------------------------------------------------------------
  let scroller: HTMLDivElement | undefined = $state();
  let scrollTop = $state(0);
  let viewportH = $state(0);
  let viewportW = $state(0);
  /** Room for a row's ref pills: half the message cell's content box (`.pills { max-width: 50% }`, `.msg` padding 18 px). */
  const pillRoom = $derived(viewportW > 0 ? Math.floor((viewportW - graphW - 18) / 2) : 0);

  const total = $derived(repo.summary.total);
  const wipCount = $derived(repo.state.files.length);
  const hasWip = $derived(wipCount > 0);
  /** The WIP row sits above row 0 inside the scroll content, so row i's top is wipH + i * ROW_H. */
  const wipH = $derived(hasWip ? ROW_H : 0);
  const range = $derived(visibleRange(scrollTop - wipH, viewportH, ROW_H, total, OVERSCAN));
  // Split into primitives so scrolling within the same slice changes nothing downstream.
  const start = $derived(range.start);
  const end = $derived(range.end);
  const indices = $derived(Array.from({ length: end - start }, (_, k) => start + k));

  const rowTop = (i: number) => wipH + i * ROW_H;

  // Fetch the pages the visible slice touches (repo.loadPage skips loaded and in-flight pages).
  $effect(() => {
    const pages = pagesFor(start, end, PAGE_SIZE);
    void repo.graphVersion;
    untrack(() => repo.showPages(pages));
  });

  // --- row ages: re-rendered once a minute so "now" and "5m" stay true -------------------------
  let now = $state(Math.floor(Date.now() / 1000));
  onMount(() => {
    const t = setInterval(() => (now = Math.floor(Date.now() / 1000)), 60_000);
    return () => clearInterval(t);
  });

  // --- selection ------------------------------------------------------------------------------
  const selectedOid = $derived(repo.selection?.kind === 'commit' ? repo.selection.oid : null);
  const wipSelected = $derived(repo.selection?.kind === 'wip');
  /** Last known index of the selected commit; verified against the oid before use. */
  let selIndex = -1;

  function selectIndex(i: number) {
    const c = repo.commitAt(i);
    if (!c) return false;
    selIndex = i;
    repo.selectCommit(c.oid);
    return true;
  }

  async function ensureLoaded(i: number) {
    if (!repo.commitAt(i)) await repo.loadPage(Math.floor(i / PAGE_SIZE));
  }

  /** Index of the selected commit from the cache or the loaded pages (synchronous); undefined when not loaded. */
  function knownIndex(oid: string): number | undefined {
    if (selIndex >= 0 && repo.commitAt(selIndex)?.oid === oid) return selIndex;
    for (const [n, rows] of repo.pages) {
      const k = rows.findIndex((r) => r.oid === oid);
      if (k >= 0) return (selIndex = n * PAGE_SIZE + k);
    }
    return undefined;
  }

  /** Index of the selected commit, asking the backend when its page is not loaded; null when none. */
  async function selectedIndex(): Promise<number | null> {
    const oid = selectedOid;
    if (!oid) return null;
    const known = knownIndex(oid);
    if (known !== undefined) return known;
    const path = repo.path;
    if (!path) return null;
    const found = await api.findCommit(path, oid).catch(() => null);
    return found === null ? null : (selIndex = found);
  }

  function keepVisible(top: number) {
    if (!scroller) return;
    const next = keepVisibleScrollTop(scroller.scrollTop, viewportH, top, ROW_H);
    if (next !== scroller.scrollTop) setScroll(next);
  }

  function setScroll(top: number) {
    if (!scroller) return;
    scroller.scrollTop = top;
    scrollTop = scroller.scrollTop;
  }

  /** Scroll so graph row `i` is visible (centred when it was off screen). Used by refs and parent-SHA links. */
  export function scrollToIndex(i: number) {
    if (!scroller || i < 0 || i >= total) return;
    setScroll(revealScrollTop(scroller.scrollTop, viewportH, rowTop(i), ROW_H));
  }

  async function move(dir: 1 | -1) {
    // Synchronous when the selection is loaded, so key repeats never read a stale index.
    let cur: number | null;
    if (wipSelected) cur = -1;
    else if (selectedOid) cur = knownIndex(selectedOid) ?? (await selectedIndex());
    else cur = null;
    let next: number;
    if (cur === null) next = hasWip ? -1 : 0;
    else next = cur + dir;
    if (next === -1 && hasWip) {
      repo.selectWip();
      selIndex = -1;
      keepVisible(0);
      return;
    }
    if (next < 0 || next >= total) return;
    if (!repo.commitAt(next)) await ensureLoaded(next);
    if (selectIndex(next)) keepVisible(rowTop(next));
  }

  // Task 13 contract: the refs column asks for a scroll via repo.reveal (revealCommit already selected it).
  let seenReveal = untrack(() => repo.reveal?.n ?? 0);
  $effect(() => {
    const r = repo.reveal;
    if (!r || r.n <= seenReveal) return;
    seenReveal = r.n;
    untrack(() => {
      selIndex = r.index;
      tick().then(() => scrollToIndex(r.index));
    });
  });

  // Switching repositories starts at the top.
  let lastPath: string | null = untrack(() => repo.path);
  $effect(() => {
    const p = repo.path;
    untrack(() => {
      if (p === lastPath) return;
      lastPath = p;
      selIndex = -1;
      setScroll(0);
    });
  });

  // --- search ---------------------------------------------------------------------------------
  let matches = $state<number[]>([]);
  let matchPos = $state(-1);
  const matchSet = $derived(new Set(matches));
  const currentMatch = $derived(matchPos >= 0 ? matches[matchPos] : -1);
  $effect(() => {
    if (!ui.searchOpen) {
      untrack(() => {
        matches = [];
        matchPos = -1;
      });
    }
  });

  async function jumpTo(i: number) {
    scrollToIndex(i);
    if (!repo.commitAt(i)) await ensureLoaded(i);
    selectIndex(i);
  }

  // --- events (delegated: rows carry data-index / data-wip, pills data-pill) ------------------
  function rowOf(target: EventTarget | null): { wip: true } | { index: number } | null {
    const el = (target as Element | null)?.closest<HTMLElement>('[data-index], [data-wip]');
    if (!el) return null;
    if (el.dataset.wip !== undefined) return { wip: true };
    return { index: Number(el.dataset.index) };
  }

  function onClick(e: MouseEvent) {
    const row = rowOf(e.target);
    if (!row) return;
    if ('wip' in row) {
      selIndex = -1;
      repo.selectWip();
    } else selectIndex(row.index);
  }

  function baseCtx(menu: MenuKind, name: string, oid: string): ActionContext {
    const head = repo.refs.head;
    return {
      menu,
      repo: repo.path ?? '',
      name,
      oid,
      current: head.branch,
      isHeadBranch: false,
      detached: head.branch === null,
      hasUpstream: false,
    };
  }

  function pillMenu(p: Pill, c: GraphCommit): { ctx: ActionContext; items: (m: ActionContext) => MenuItem[] } {
    if (p.tag) return { ctx: baseCtx('tag', p.name, c.oid), items: tagMenu };
    if (p.local) {
      const b = repo.refs.local.find((x) => x.name === p.name);
      return {
        ctx: { ...baseCtx('branch', p.name, c.oid), isHeadBranch: p.isHead, hasUpstream: !!b?.upstream },
        items: branchMenu,
      };
    }
    return { ctx: baseCtx('remoteBranch', p.name, c.oid), items: remoteBranchMenu };
  }

  function onContextMenu(e: MouseEvent) {
    const row = rowOf(e.target);
    if (!row) return;
    e.preventDefault();
    if ('wip' in row) {
      selIndex = -1;
      repo.selectWip();
      return;
    }
    const c = repo.commitAt(row.index);
    if (!c || !repo.path) return;
    selectIndex(row.index);
    const pillEl = (e.target as Element).closest<HTMLElement>('[data-pill]');
    const pill = pillEl ? rowPills(c.refs)[Number(pillEl.dataset.pill)] : undefined;
    const { ctx, items } = pill ? pillMenu(pill, c) : { ctx: baseCtx('commit', c.shortOid, c.oid), items: commitMenu };
    openMenu(e.clientX, e.clientY, items(ctx), (id) => performMenuAction(id, ctx));
  }

  // --- pill hover: a truncated pill shown in full, or every pill of a row with a "+N" chip ------
  const PEEK_DELAY = 120;
  /** How long a pill list stays up after the pointer leaves the row's pills, so it can move into the list. */
  const PEEK_GRACE = 200;
  let peek = $state<{
    pills: Pill[]; color: string; mode: 'expand' | 'list'; anchor: DOMRect; index: number;
    /** The pill whose context menu is open: the list stays up, that pill lit, until the menu closes. */
    menuFor?: number;
  } | null>(null);
  let peekEl: Element | null = null;
  let peekTimer: ReturnType<typeof setTimeout> | undefined;
  let graceTimer: ReturnType<typeof setTimeout> | undefined;

  function hidePeek() {
    clearTimeout(peekTimer);
    clearTimeout(graceTimer);
    peekEl = null;
    peek = null;
  }

  /** Keep the shown "+N" list up a moment (the pointer may be heading into it), or hide anything else now. */
  function leavePeek() {
    if (peek?.menuFor !== undefined) return;
    if (peek?.mode !== 'list') return hidePeek();
    clearTimeout(graceTimer);
    graceTimer = setTimeout(hidePeek, PEEK_GRACE);
  }

  function keepPeek() {
    clearTimeout(graceTimer);
  }

  const inPeek = (n: EventTarget | null) => !!(n as Element | null)?.closest?.('[data-peek]');

  function showPeek(el: HTMLElement) {
    const row = rowOf(el);
    const index = row && 'index' in row ? row.index : -1;
    const c = index >= 0 ? repo.commitAt(index) : undefined;
    if (!c || !el.isConnected) return;
    const pills = rowPills(c.refs);
    const color = laneColor(c.color);
    const anchor = el.getBoundingClientRect();
    if (el.dataset.overflow !== undefined) {
      peek = { pills, color, mode: 'list', anchor, index };
      return;
    }
    const name = el.querySelector('.name');
    const pill = pills[Number(el.dataset.pill)];
    if (pill && name && name.scrollWidth > name.clientWidth) peek = { pills: [pill], color, mode: 'expand', anchor, index };
  }

  function onPointerOver(e: PointerEvent) {
    if (peek?.menuFor !== undefined) return;
    // In a row with hidden pills, any pill or the chip opens the list of them all.
    const hit = (e.target as Element).closest<HTMLElement>('[data-pill], .more');
    const el = hit?.closest<HTMLElement>('[data-overflow]') ?? hit;
    if (el && el === peekEl) return keepPeek();
    if (!el && peek?.mode === 'list') return leavePeek();
    hidePeek();
    if (!el) return;
    peekEl = el;
    peekTimer = setTimeout(() => showPeek(el), PEEK_DELAY);
  }

  function onPointerOut(e: PointerEvent) {
    if (!peekEl || peekEl.contains(e.relatedTarget as Node | null)) return;
    if (inPeek(e.relatedTarget)) keepPeek();
    else leavePeek();
  }

  function onPeekLeave(e: PointerEvent) {
    if (peek?.menuFor !== undefined) return;
    if (!peekEl?.contains(e.relatedTarget as Node | null)) hidePeek();
  }

  /** The pill under the pointer in a pill list, with its row's commit. */
  function peekPill(e: MouseEvent): { pill: Pill; index: number; c: GraphCommit } | null {
    const el = (e.target as Element).closest<HTMLElement>('[data-pill]');
    const pill = el && peek ? peek.pills[Number(el.dataset.pill)] : undefined;
    const c = peek ? repo.commitAt(peek.index) : undefined;
    return pill && c && peek ? { pill, index: peek.index, c } : null;
  }

  function onPeekClick(e: MouseEvent) {
    const hit = peekPill(e);
    hidePeek();
    if (hit) selectIndex(hit.index);
  }

  function onPeekContextMenu(e: MouseEvent) {
    e.preventDefault();
    const hit = peekPill(e);
    if (!hit || !repo.path || !peek) return hidePeek();
    clearTimeout(graceTimer);
    peek = { ...peek, menuFor: peek.pills.indexOf(hit.pill) };
    selectIndex(hit.index);
    const { ctx, items } = pillMenu(hit.pill, hit.c);
    openMenu(e.clientX, e.clientY, items(ctx), (id) => performMenuAction(id, ctx));
  }

  // A list kept up for its pill's menu goes when the menu closes.
  $effect(() => {
    if (contextMenu.current) return;
    untrack(() => {
      if (peek?.menuFor !== undefined) hidePeek();
    });
  });

  // The anchor rect goes stale when the list scrolls or changes, or another repository is shown.
  $effect(() => {
    void scrollTop;
    void repo.path;
    void repo.graphVersion;
    untrack(() => {
      if (peekEl) hidePeek();
    });
  });

  // Focus ring on the selected row only after keyboard use; a click focuses the scroller programmatically.
  let keyboardNav = $state(false);

  function onKeydown(e: KeyboardEvent) {
    keyboardNav = true;
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      move(e.key === 'ArrowDown' ? 1 : -1);
    }
  }

  function onPointerDown() {
    // Clicking a row gives the list keyboard focus without scrolling.
    if (scroller && document.activeElement !== scroller) scroller.focus({ preventScroll: true });
  }

  // --- column resize --------------------------------------------------------------------------
  function startDrag(e: PointerEvent) {
    e.preventDefault();
    const startX = e.clientX;
    const startW = graphW;
    const target = e.currentTarget as HTMLElement;
    target.setPointerCapture(e.pointerId);
    const onMove = (ev: PointerEvent) => {
      const w = Math.round(startW + ev.clientX - startX);
      graphCap = Math.min(GRAPH_MAX, Math.max(GRAPH_MIN, w));
    };
    const up = () => {
      target.removeEventListener('pointermove', onMove);
      target.removeEventListener('pointerup', up);
      target.removeEventListener('pointercancel', up);
    };
    target.addEventListener('pointermove', onMove);
    target.addEventListener('pointerup', up);
    target.addEventListener('pointercancel', up);
  }
</script>

<div class="graphview" style:--graph-w="{graphW}px">
  <div class="header">
    <div class="h">Graph</div>
    <div class="h">
      Commit message
      <span class="grip" role="separator" aria-orientation="vertical" aria-label="Resize graph column" onpointerdown={startDrag}></span>
    </div>
  </div>

  <div
    class="scroller"
    class:kbd={keyboardNav}
    bind:this={scroller}
    bind:clientHeight={viewportH}
    bind:clientWidth={viewportW}
    onscroll={() => (scrollTop = scroller?.scrollTop ?? 0)}
    onclick={onClick}
    oncontextmenu={onContextMenu}
    onkeydown={onKeydown}
    onpointerover={onPointerOver}
    onpointerout={onPointerOut}
    onpointerdown={() => {
      keyboardNav = false;
      hidePeek();
      onPointerDown();
    }}
    role="listbox"
    aria-label="Commit history"
    tabindex="0"
  >
    <div class="content" style:height="{wipH + total * ROW_H}px">
      {#if hasWip}
        <div class="wiprow">
          <GraphRow index={-1} wipCount={wipCount} wipLinked={repo.summary.headIndex !== null} selected={wipSelected} />
        </div>
      {/if}
      <div class="slice" style:transform="translateY({rowTop(start)}px)">
        {#each indices as i (i)}
          {@const c = repo.commitAt(i)}
          <GraphRow
            index={i}
            commit={c}
            selected={!!c && c.oid === selectedOid}
            match={matchSet.has(i)}
            current={i === currentMatch}
            age={c ? shortAge(c.authorTime, now) : ''}
            headIndex={repo.summary.headIndex}
            hasWip={hasWip}
            pillRoom={pillRoom}
          />
        {/each}
      </div>
    </div>
    {#if repo.loading && total === 0}
      <!-- A repository is opening: placeholder rows, shown only if that takes a moment. -->
      <div class="loadingrows skeleton-delayed" aria-hidden="true">
        {#each { length: Math.max(1, Math.ceil(viewportH / ROW_H)) } as _, i (i)}
          <GraphRow index={i} />
        {/each}
      </div>
    {/if}
    {#if total === 0 && !hasWip && !repo.loading}
      <p class="empty">No commits yet</p>
    {/if}
  </div>

  {#if peek}
    <PillPeek
      pills={peek.pills}
      color={peek.color}
      mode={peek.mode}
      anchor={peek.anchor}
      active={peek.menuFor}
      role={peek.mode === 'list' ? 'group' : undefined}
      aria-label={peek.mode === 'list' ? 'More branches and tags' : undefined}
      aria-hidden={peek.mode === 'expand' ? 'true' : undefined}
      onpointerenter={keepPeek}
      onpointerleave={onPeekLeave}
      onclick={onPeekClick}
      oncontextmenu={onPeekContextMenu}
    />
  {/if}

  {#if ui.searchOpen}
    <div class="searchwrap">
      <SearchBar bind:matches bind:pos={matchPos} onjump={jumpTo} />
    </div>
  {/if}
</div>

<style>
  .graphview {
    position: relative;
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .header {
    display: grid;
    flex: none;
    grid-template-columns: var(--graph-w) minmax(0, 1fr);
    height: 28px;
    border-bottom: 1px solid var(--line);
    background: var(--panel);
  }
  .h {
    position: relative;
    display: flex;
    align-items: center;
    min-width: 0;
    padding: 0 10px;
    overflow: hidden;
    color: var(--muted);
    font-size: var(--fs-xs);
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    white-space: nowrap;
  }
  .h + .h {
    border-left: 1px solid var(--line);
  }
  .grip {
    position: absolute;
    top: 0;
    bottom: 0;
    left: -3px;
    width: 6px;
    cursor: col-resize;
    touch-action: none;
    z-index: 1;
  }
  .grip:hover {
    background: var(--accent);
  }
  .scroller {
    position: relative;
    flex: 1;
    min-height: 0;
    overflow-x: hidden;
    overflow-y: auto;
    overscroll-behavior: contain;
    outline: none;
  }
  /* Keyboard focus lives on the scroller; show it on the selected row. */
  .scroller.kbd:focus :global(.row.selected) {
    box-shadow: inset 0 0 0 2px var(--focus);
  }
  .content {
    position: relative;
    min-height: 100%;
  }
  .wiprow {
    position: absolute;
    top: 0;
    right: 0;
    left: 0;
  }
  .slice {
    position: absolute;
    top: 0;
    right: 0;
    left: 0;
    will-change: transform;
  }
  .loadingrows {
    position: absolute;
    inset: 0;
    overflow: hidden;
    pointer-events: none;
  }
  .empty {
    position: absolute;
    inset: 40% 0 auto;
    margin: 0;
    color: var(--muted);
    font-size: var(--fs-md);
    text-align: center;
  }
  /* Floats over the right end of the column header row, so it never covers a commit. */
  .searchwrap {
    position: absolute;
    top: 3px;
    right: 14px;
    z-index: 5;
  }
</style>
