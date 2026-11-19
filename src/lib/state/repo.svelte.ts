import { SvelteMap } from 'svelte/reactivity';
import * as api from '../api';
import type { ChangeKind, GraphCommit, GraphSummary, Refs, RepoState } from '../types';
import { pagesWithin } from '../util/graphGeometry';
import { graphRefsChanged } from '../util/refs';
import { leavesConflictView } from '../util/liveDiff';
import { toasts } from './toasts.svelte';

export const PAGE_SIZE = 200;

/** `outsideGraph`: a commit the graph does not list (a stash); details and diffs still work for it. */
export type Selection = { kind: 'commit'; oid: string; outsideGraph?: boolean } | { kind: 'wip' } | null;
export type Centre =
  | { kind: 'graph' }
  | {
      kind: 'diff';
      path: string;
      origPath: string | null;
      source: import('../types').DiffSource;
      view: 'diff' | 'file';
    };

const emptyState = (): RepoState => ({ head: { branch: null, oid: null }, operation: null, files: [] });
const emptyRefs = (): Refs => ({ head: { branch: null, oid: null }, local: [], remote: [], remotes: [], tags: [], stashes: [], worktrees: [] });
const emptySummary = (): GraphSummary => ({ total: 0, maxLanes: 0, headIndex: null });

/**
 * What a repository looked like when you left it, so switching back shows it at once. Only the
 * graph pages in view are kept (others reload from the backend's graph cache as you scroll).
 */
interface Snapshot {
  refs: Refs;
  state: RepoState;
  summary: GraphSummary;
  pages: Map<number, GraphCommit[]>;
  shown: number[];
  selection: Selection;
  centre: Centre;
  scrollTop: number;
}

function report(title: string, e: unknown) {
  const err = e as { message?: string; stderr?: string };
  toasts.push({ kind: 'error', title, detail: err?.stderr || err?.message || String(e) });
}

/** State of the currently selected repository. */
class RepoStore {
  path = $state<string | null>(null);
  loading = $state(false);
  refs = $state<Refs>(emptyRefs());
  state = $state<RepoState>(emptyState());
  summary = $state<GraphSummary>(emptySummary());
  /** Loaded graph pages keyed by page number (page size PAGE_SIZE). */
  pages = new SvelteMap<number, GraphCommit[]>();
  selection = $state<Selection>(null);
  centre = $state<Centre>({ kind: 'graph' });
  /** Bumped on every graph reload so views can reset scroll/caches. */
  graphVersion = $state(0);
  /** Bumped (monotonically) whenever the working state is refreshed, so live diffs re-fetch. */
  stateVersion = $state(0);
  /** Request for the graph to scroll to a row; `n` makes repeated requests for one index distinct. */
  reveal = $state<{ index: number; n: number } | null>(null);
  /** The graph's scroll offset. GraphView keeps it current and applies it on a switch; not reactive. */
  scrollTop = 0;

  #token = 0;
  #inflight = new Set<number>();
  /** Pages the graph view shows; a reload re-fetches these before swapping, so the rows never go blank. */
  #shown: number[] = [];
  #reloadSeq = 0;
  /** Repositories switched away from this session, by path. */
  #saved = new Map<string, Snapshot>();

  /**
   * Switch to `path` (null clears). A repository seen before this session comes back as it was left,
   * then is checked for changes in the background; otherwise its refs, state and graph summary load.
   */
  async open(path: string | null) {
    this.#token++;
    // A repository still loading has nothing worth keeping.
    if (this.path && !this.loading) this.#saved.set(this.path, this.#snapshot());
    this.path = path;
    this.#inflight.clear();
    this.loading = false;
    const saved = path ? this.#saved.get(path) : undefined;
    if (path && saved) {
      this.#saved.delete(path);
      this.#restore(saved);
      await this.#revalidate(path);
      return;
    }
    this.pages.clear();
    this.#shown = [];
    this.selection = null;
    this.centre = { kind: 'graph' };
    this.refs = emptyRefs();
    this.state = emptyState();
    this.summary = emptySummary();
    this.scrollTop = 0;
    if (!path) return;
    this.loading = true;
    const token = this.#token;
    try {
      const [refs, st, summary] = await Promise.all([api.listRefs(path), api.repoState(path), api.loadGraph(path)]);
      if (token !== this.#token) return;
      this.refs = refs;
      this.state = st;
      this.summary = summary;
      this.graphVersion++;
    } catch (e) {
      if (token === this.#token) report('Could not open repository', e);
    } finally {
      if (token === this.#token) this.loading = false;
    }
  }

  /** Drop what is kept for repositories that are no longer open. */
  retain(open: string[]) {
    for (const p of [...this.#saved.keys()]) if (!open.includes(p)) this.#saved.delete(p);
  }

  #snapshot(): Snapshot {
    const pages = new Map<number, GraphCommit[]>();
    for (const n of this.#shown) {
      const rows = this.pages.get(n);
      if (rows) pages.set(n, rows);
    }
    return {
      refs: this.refs,
      state: this.state,
      summary: this.summary,
      pages,
      shown: [...this.#shown],
      selection: this.selection,
      centre: this.centre,
      scrollTop: this.scrollTop,
    };
  }

  #restore(s: Snapshot) {
    this.refs = s.refs;
    this.state = s.state;
    this.summary = s.summary;
    this.pages.clear();
    for (const [n, rows] of s.pages) this.pages.set(n, rows);
    this.#shown = s.shown;
    this.selection = s.selection;
    this.centre = s.centre;
    this.scrollTop = s.scrollTop;
    this.graphVersion++;
    this.stateVersion++;
  }

  /**
   * After switching back: re-read the refs and working state (both cheap), and rebuild the graph
   * only when something it is drawn from moved while the repository was out of view.
   */
  async #revalidate(path: string) {
    const token = this.#token;
    try {
      const [refs, st] = await Promise.all([api.listRefs(path), api.repoState(path)]);
      if (token !== this.#token) return;
      const graphStale = graphRefsChanged(this.refs, refs);
      this.refs = refs;
      this.#setState(st);
      if (graphStale) await this.reloadGraph();
    } catch (e) {
      if (token === this.#token) report('Refresh failed', e);
    }
  }

  /**
   * 'Git' (refs/HEAD/index changed): reload refs, repo state and graph summary, clearing pages.
   * 'Worktree': reload repo state only.
   */
  async refresh(kinds: ChangeKind[]) {
    const path = this.path;
    if (!path) return;
    const token = this.#token;
    try {
      if (kinds.includes('Git')) {
        const [refs, st] = await Promise.all([api.listRefs(path), api.repoState(path)]);
        if (token !== this.#token) return;
        this.refs = refs;
        this.#setState(st);
        await this.reloadGraph();
      } else if (kinds.includes('Worktree')) {
        const st = await api.repoState(path);
        if (token === this.#token) this.#setState(st);
      }
    } catch (e) {
      if (token === this.#token) report('Refresh failed', e);
    }
  }

  #setState(st: RepoState) {
    // A finished merge/rebase/... leaves nothing to show in the conflict view.
    if (leavesConflictView(this.state.operation, st.operation, this.centre)) this.centre = { kind: 'graph' };
    this.state = st;
    this.stateVersion++;
  }

  /**
   * Reload the graph summary and pages; the selection (by oid) is kept. The old rows stay on screen
   * until the new summary and the pages in view have arrived, then everything swaps at once.
   */
  async reloadGraph() {
    const path = this.path;
    if (!path) return;
    const token = this.#token;
    const seq = ++this.#reloadSeq;
    const current = () => token === this.#token && seq === this.#reloadSeq;
    const summary = await api.loadGraph(path);
    if (!current()) return;
    let fresh = new Map<number, GraphCommit[]>();
    try {
      const keep = pagesWithin(this.#shown, summary.total, PAGE_SIZE);
      const rows = await Promise.all(keep.map((n) => api.graphPage(path, n * PAGE_SIZE, PAGE_SIZE)));
      fresh = new Map(keep.map((n, i) => [n, rows[i]]));
    } catch {
      // The view loads (and reports) whatever is missing after the swap.
    }
    if (!current()) return;
    this.summary = summary;
    this.pages.clear();
    this.#inflight.clear();
    for (const [n, rows] of fresh) this.pages.set(n, rows);
    this.graphVersion++;
  }

  /** Load the pages the graph view shows and remember them for the next reload. */
  showPages(ns: number[]) {
    this.#shown = ns;
    for (const n of ns) this.loadPage(n);
  }

  /** Ensure page `n` is loaded. */
  async loadPage(n: number) {
    const path = this.path;
    if (!path || this.pages.has(n) || this.#inflight.has(n)) return;
    const token = this.#token;
    const version = this.graphVersion;
    this.#inflight.add(n);
    try {
      const rows = await api.graphPage(path, n * PAGE_SIZE, PAGE_SIZE);
      if (token === this.#token && version === this.graphVersion) this.pages.set(n, rows);
    } catch (e) {
      if (token === this.#token) report('Could not load history', e);
    } finally {
      this.#inflight.delete(n);
    }
  }

  /** Commit at absolute graph index, if its page is loaded. */
  commitAt(index: number): GraphCommit | undefined {
    return this.pages.get(Math.floor(index / PAGE_SIZE))?.[index % PAGE_SIZE];
  }

  selectCommit(oid: string) {
    this.selection = { kind: 'commit', oid };
    this.centre = { kind: 'graph' };
  }
  /** Find `oid` in the graph, select it and ask the graph to scroll to it. Returns false when it is not in the graph. */
  async revealCommit(oid: string): Promise<boolean> {
    const path = this.path;
    if (!path) return false;
    try {
      const index = await api.findCommit(path, oid);
      if (path !== this.path || index === null) return false;
      this.selectCommit(oid);
      this.reveal = { index, n: (this.reveal?.n ?? 0) + 1 };
      return true;
    } catch (e) {
      report('Could not locate commit', e);
      return false;
    }
  }
  /**
   * Select a stash commit. Stashes are not in the graph, so when it cannot be revealed it is
   * still selected (marked outsideGraph) so the details panel shows its contents.
   */
  async selectStash(oid: string) {
    const path = this.path;
    if (await this.revealCommit(oid)) return;
    if (path !== this.path) return;
    this.selection = { kind: 'commit', oid, outsideGraph: true };
    this.centre = { kind: 'graph' };
  }
  selectWip() {
    this.selection = { kind: 'wip' };
    this.centre = { kind: 'graph' };
  }
  showGraph() {
    this.centre = { kind: 'graph' };
  }
}

export const repo = new RepoStore();
