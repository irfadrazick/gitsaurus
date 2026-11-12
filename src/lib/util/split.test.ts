import { expect, it } from 'vitest';
import type { DiffLine, Hunk } from '../types';
import { toSplitRows } from './split';

const l = (kind: DiffLine['kind'], text: string): DiffLine => ({ kind, oldNo: null, newNo: null, text });

it('pairs dels with adds', () => {
  const [c1, d1, d2, a1, c2] = [l('context', 'c1'), l('del', 'd1'), l('del', 'd2'), l('add', 'a1'), l('context', 'c2')];
  const h: Hunk = { header: '', oldStart: 1, oldLines: 4, newStart: 1, newLines: 3, lines: [c1, d1, d2, a1, c2] };
  expect(toSplitRows(h)).toEqual([
    { left: c1, right: c1 },
    { left: d1, right: a1 },
    { left: d2 },
    { left: c2, right: c2 },
  ]);
});

it('lone adds go right only', () => {
  const a = l('add', 'x');
  expect(toSplitRows({ header: '', oldStart: 0, oldLines: 0, newStart: 1, newLines: 1, lines: [a] })).toEqual([{ right: a }]);
});
