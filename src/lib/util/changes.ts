import type { Change, ChangedFile } from '../types';

const ORDER: [Change, string][] = [
  ['modified', 'modified'],
  ['added', 'added'],
  ['deleted', 'deleted'],
  ['renamed', 'renamed'],
  ['copied', 'copied'],
  ['typeChange', 'type changed'],
];

/** "3 modified · 4 added" (fixed kind order, zero counts omitted). */
export function changeSummary(files: ChangedFile[]): string {
  return ORDER.map(([kind, label]) => [files.filter((f) => f.change === kind).length, label] as const)
    .filter(([n]) => n > 0)
    .map(([n, label]) => `${n} ${label}`)
    .join(' · ');
}
