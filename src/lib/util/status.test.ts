import { expect, it } from 'vitest';
import type { FileStatus } from '../types';
import { splitStatus } from './status';

const s = (path: string, index: string, worktree: string, extra: Partial<FileStatus> = {}): FileStatus => ({
  path, origPath: null, index, worktree, conflicted: false, untracked: false, ...extra,
});

it('splits files', () => {
  const files = [
    s('both', 'M', 'M'),
    s('staged', 'A', '.'),
    s('wt', '.', 'M'),
    s('new', '.', '?', { untracked: true }),
    s('conf', 'U', 'U', { conflicted: true }),
  ];
  const r = splitStatus(files);
  expect(r.conflicted.map((f) => f.path)).toEqual(['conf']);
  expect(r.staged.map((f) => f.path)).toEqual(['both', 'staged']);
  expect(r.unstaged.map((f) => f.path)).toEqual(['both', 'wt', 'new']);
});

it('treats the backend\'s "." as unchanged', () => {
  const r = splitStatus([s('wt', '.', 'M'), s('idx', 'A', '.'), s('del', 'D', '.'), s('rm', '.', 'D')]);
  expect(r.staged.map((f) => f.path)).toEqual(['idx', 'del']);
  expect(r.unstaged.map((f) => f.path)).toEqual(['wt', 'rm']);
});
