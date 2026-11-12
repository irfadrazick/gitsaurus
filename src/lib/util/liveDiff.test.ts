import { expect, it } from 'vitest';
import type { DiffSource, Operation } from '../types';
import { isLiveSource, leavesConflictView } from './liveDiff';

it('working-state sources are live; commit diffs are not', () => {
  expect(isLiveSource({ kind: 'unstaged' })).toBe(true);
  expect(isLiveSource({ kind: 'staged' })).toBe(true);
  expect(isLiveSource({ kind: 'untracked' })).toBe(true);
  expect(isLiveSource({ kind: 'workingFile' })).toBe(true);
  expect(isLiveSource({ kind: 'commit', oid: 'abc' })).toBe(false);
});

const op: Operation = { kind: 'merge', step: null, total: null, oursLabel: 'main', theirsLabel: 'feat' };
const diff = (source: DiffSource) => ({ kind: 'diff' as const, path: 'f', origPath: null, source, view: 'diff' as const });

it('returns to the graph when the operation ends while a conflicted file is shown', () => {
  expect(leavesConflictView(op, null, diff({ kind: 'workingFile' }))).toBe(true);
});

it('stays put otherwise', () => {
  // Operation still running, or never was (a stash conflict).
  expect(leavesConflictView(op, op, diff({ kind: 'workingFile' }))).toBe(false);
  expect(leavesConflictView(null, null, diff({ kind: 'workingFile' }))).toBe(false);
  // Other diffs, or the graph, are unaffected.
  expect(leavesConflictView(op, null, diff({ kind: 'staged' }))).toBe(false);
  expect(leavesConflictView(op, null, { kind: 'graph' })).toBe(false);
});
