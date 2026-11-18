import type { Pill, RefLabel } from '../types';

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

const pillKind = (p: Pill) =>
  p.tag ? 'tag' : p.local && p.remote ? 'local + remote' : p.local ? 'local' : 'remote';

/** Tooltip listing every pill of a row, one per line. */
export function pillTitle(pills: Pill[]): string {
  return pills.map((p) => `${p.isHead ? '✓ ' : ''}${p.name} (${pillKind(p)})`).join('\n');
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
