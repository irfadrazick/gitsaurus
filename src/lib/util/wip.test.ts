import { expect, it } from 'vitest';
import type { FileStatus } from '../types';
import { bannerText, canCommit, commitMessageParts, toWipFile } from './wip';

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
  const op = (kind: 'merge' | 'rebase' | 'cherryPick' | 'revert', step: number | null = null, total: number | null = null) =>
    ({ kind, step, total, oursLabel: '', theirsLabel: '' });
  expect(bannerText(op('merge'), 2)).toBe('Merge conflict in 2 files');
  expect(bannerText(op('merge'), 1)).toBe('Merge conflict in 1 file');
  expect(bannerText(op('merge'), 0)).toBe('Merge in progress: all conflicts resolved');
  expect(bannerText(op('rebase', 3, 7), 2)).toBe('Rebase step 3/7: 2 conflicted files');
  expect(bannerText(op('rebase', 3, 7), 0)).toBe('Rebase step 3/7: all conflicts resolved');
  expect(bannerText(op('rebase'), 1)).toBe('Rebase: 1 conflicted file');
  expect(bannerText(op('cherryPick'), 1)).toBe('Cherry-pick conflict in 1 file');
  expect(bannerText(op('revert'), 2)).toBe('Revert conflict in 2 files');
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
