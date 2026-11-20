import type { Pill, RefLabel, Refs } from '../types';

/** Collapse ref labels into pills: a remote `origin/x` merges into local `x` when both exist. */
export function groupPills(refs: RefLabel[]): Pill[] {
  const pills: Pill[] = [];
  const byName = new Map<string, Pill>();
  const add = (name: string, make: () => Pill) => {
    let p = byName.get(name);
    if (!p) {
      p = make();
      byName.set(name, p);
      pills.push(p);
    }
    return p;
  };
  const blank = (name: string): Pill => ({ name, local: false, remote: false, tag: false, isHead: false });
  for (const r of refs) {
    if (r.kind === 'local') {
      const p = add(r.name, () => blank(r.name));
      p.local = true;
      p.isHead ||= r.isHead;
    }
  }
  for (const r of refs) {
    if (r.kind === 'remote') {
      const short = r.name.slice(r.name.indexOf('/') + 1);
      const local = byName.get(short);
      if (local?.local) local.remote = true;
      else add(r.name, () => blank(r.name)).remote = true;
    } else if (r.kind === 'tag') {
      add(`tag:${r.name}`, () => ({ ...blank(r.name), tag: true }));
    }
  }
  return pills;
}

/** Pills for a graph row: groupPills order, with the checked-out branch moved first. */
export function rowPills(refs: RefLabel[]): Pill[] {
  const pills = groupPills(refs);
  const head = pills.findIndex((p) => p.isHead);
  if (head > 0) pills.unshift(...pills.splice(head, 1));
  return pills;
}

/** Most pills shown inline on a graph row; the rest collapse into a "+N" chip. */
export const MAX_PILLS_SHOWN = 2;
/** Gap between pills, in px (matches `.pills { gap }` in RefPills.svelte). */
export const PILL_GAP = 4;

/**
 * How many of a row's pills to show in full in `room` px: up to MAX_PILLS_SHOWN when they and the
 * "+N" chip for the rest fit, else just the first (it may truncate). `widths` are the pills' natural
 * widths, `moreWidth(n)` is the width of a "+n" chip; a `room` of 0 or less (not measured yet) shows the most.
 */
export function pillsShown(widths: number[], moreWidth: (n: number) => number, room: number): number {
  const n = widths.length;
  const most = Math.min(n, MAX_PILLS_SHOWN);
  if (room <= 0) return most;
  for (let k = most; k > 1; k--) {
    let total = widths.slice(0, k).reduce((a, w) => a + w, 0) + PILL_GAP * (k - 1);
    if (n > k) total += PILL_GAP + moreWidth(n - k);
    if (total <= room) return k;
  }
  return Math.min(n, 1);
}

/** What the commit graph is drawn from: it reads the log of HEAD, the branches, the remote branches and the tags. */
function graphKey(r: Refs): string {
  return JSON.stringify([
    r.head.branch,
    r.head.oid,
    r.local.map((b) => [b.name, b.oid]),
    r.remote.map((b) => [b.fullName, b.oid]),
    r.tags.map((t) => [t.name, t.oid]),
  ]);
}

/** Whether the graph built from `before` may differ from one built from `after` (stashes and worktrees aren't drawn). */
export function graphRefsChanged(before: Refs, after: Refs): boolean {
  return graphKey(before) !== graphKey(after);
}

/** A branch list as folders: `release/v4.7` is the ref `v4.7` inside the folder `release`. */
export type RefNode<T> =
  | { kind: 'folder'; name: string; path: string; children: RefNode<T>[] }
  | { kind: 'ref'; name: string; item: T };

/**
 * Nests refs by the `/`-separated parts of their names. Order is kept: each folder sits where its
 * first ref came, so an alphabetical list stays alphabetical.
 */
export function refTree<T>(items: T[], nameOf: (t: T) => string): RefNode<T>[] {
  const root: RefNode<T>[] = [];
  const folders = new Map<string, RefNode<T>[]>();
  for (const item of items) {
    const parts = nameOf(item).split('/');
    let level = root;
    for (let i = 0; i < parts.length - 1; i++) {
      const path = parts.slice(0, i + 1).join('/');
      let children = folders.get(path);
      if (!children) {
        children = [];
        folders.set(path, children);
        level.push({ kind: 'folder', name: parts[i], path, children });
      }
      level = children;
    }
    level.push({ kind: 'ref', name: parts[parts.length - 1], item });
  }
  return root;
}

/** The refs of a tree in the order they are listed, leaving out the contents of closed folders. */
export function visibleRefs<T>(nodes: RefNode<T>[], isOpen: (path: string) => boolean): T[] {
  const out: T[] = [];
  const walk = (level: RefNode<T>[]) => {
    for (const n of level) {
      if (n.kind === 'ref') out.push(n.item);
      else if (isOpen(n.path)) walk(n.children);
    }
  };
  walk(nodes);
  return out;
}
