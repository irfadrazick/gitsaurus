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
