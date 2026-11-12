import type { DiffLine, Hunk, SplitRow } from '../types';

/** Side-by-side rows: consecutive dels pair with the following adds. */
export function toSplitRows(h: Hunk): SplitRow[] {
  const rows: SplitRow[] = [];
  let dels: DiffLine[] = [];
  let adds: DiffLine[] = [];
  const flush = () => {
    const n = Math.max(dels.length, adds.length);
    for (let i = 0; i < n; i++) {
      const row: SplitRow = {};
      if (dels[i]) row.left = dels[i];
      if (adds[i]) row.right = adds[i];
      rows.push(row);
    }
    dels = [];
    adds = [];
  };
  for (const l of h.lines) {
    if (l.kind === 'del') {
      if (adds.length) flush();
      dels.push(l);
    } else if (l.kind === 'add') adds.push(l);
    else {
      flush();
      rows.push({ left: l, right: l });
    }
  }
  flush();
  return rows;
}
