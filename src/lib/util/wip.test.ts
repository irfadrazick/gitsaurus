import { expect, it } from 'vitest';
import type { FileStatus } from '../types';
import { bannerText, canCommit, commitMessageParts, conflictSides, toWipFile } from './wip';

const s = (path: string, index: string, worktree: string, extra: Partial<FileStatus> = {}): FileStatus => ({
  path, origPath: null, index, worktree, conflicted: false, untracked: false, ...extra,
});

it('maps the index side', () => {
  expect(toWipFile(s('a', 'A', '.'), 'index').change).toBe('added');
  expect(toWipFile(s('a', 'M', 'M'), 'index').change).toBe('modified');
  expect(toWipFile(s('a', 'D', '.'), 'index').change).toBe('deleted');
  expect(toWipFile(s('b', 'R', '.', { origPath: 'a' }), 'index')).toEqual({ path: 'b', origPath: 'a', change: 'renamed' });
  expect(toWipFile(s('a', 'C', '.'), 'index').change).toBe('copied');
  expect(toWipFile(s('a', 'T', '.'), 'index').change).toBe('typeChange');
});

it('maps the worktree side, untracked and conflicts', () => {
  expect(toWipFile(s('a', 'M', 'M'), 'worktree').change).toBe('modified');
  expect(toWipFile(s('a', '.', 'D'), 'worktree').change).toBe('deleted');
  expect(toWipFile(s('n', '.', '?', { untracked: true }), 'worktree').change).toBe('added');
  const c = toWipFile(s('c', 'U', 'U', { conflicted: true }), 'worktree');
  expect(c.conflict).toBe(true);
  expect(c.change).toBe('modified');
});

it('banner text', () => {
  type K = 'merge' | 'rebase' | 'cherryPick' | 'revert';
  const op = (kind: K, step: number | null = null, total: number | null = null, oursLabel = '', theirsLabel = '') =>
    ({ kind, step, total, oursLabel, theirsLabel });
  // Names both sides when the backend knows them.
  expect(bannerText(op('merge', null, null, 'main', 'dark-theme'), 1)).toBe('Merging dark-theme into main: 1 conflicted file');
  expect(bannerText(op('merge', null, null, 'main', 'dark-theme'), 0)).toBe('Merging dark-theme into main: all conflicts resolved');
  expect(bannerText(op('rebase', 3, 7, 'main', 'feat'), 2)).toBe('Rebasing feat onto main, step 3/7: 2 conflicted files');
  expect(bannerText(op('cherryPick', null, null, 'main', 'a1b2c3d'), 1)).toBe('Cherry-picking a1b2c3d: 1 conflicted file');
  expect(bannerText(op('revert', null, null, 'main', 'a1b2c3d'), 2)).toBe('Reverting a1b2c3d: 2 conflicted files');
  // Falls back to the operation name when a side is unknown.
  expect(bannerText(op('merge'), 2)).toBe('Merge: 2 conflicted files');
  expect(bannerText(op('merge'), 0)).toBe('Merge: all conflicts resolved');
  expect(bannerText(op('rebase', 3, 7), 0)).toBe('Rebase, step 3/7: all conflicts resolved');
  expect(bannerText(op('rebase'), 1)).toBe('Rebase: 1 conflicted file');
});

it('canCommit', () => {
  expect(canCommit('x', 1, false)).toBe(true);
  expect(canCommit('  ', 1, false)).toBe(false);
  expect(canCommit('x', 0, false)).toBe(false);
  expect(canCommit('x', 0, true)).toBe(true);
  expect(canCommit('', 0, true)).toBe(false);
});

it('commitMessageParts', () => {
  const r = commitMessageParts('  sum  ', '\n  \n    indented\n- item  \n\n ');
  expect(r.summary).toBe('sum');
  expect(r.description).toBe('    indented\n- item');
  expect(commitMessageParts('s', '  \n\t\n').description).toBe('');
});

it('conflict side names', () => {
  const op = { kind: 'merge' as const, step: null, total: null, oursLabel: 'main', theirsLabel: 'feat' };
  expect(conflictSides(op)).toEqual({ ours: '"main"', theirs: '"feat"' });
  // No operation: the conflict came from applying a stash.
  expect(conflictSides(null)).toEqual({ ours: 'mine (current)', theirs: 'stash' });
});
