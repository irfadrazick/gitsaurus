import { expect, it } from 'vitest';
import type { Worktree } from '../types';
import { worktreeMenu } from './menus';
import { checkedOutElsewhere, isDirtyWorktreeError, samePath, worktreeLabel } from './worktrees';

const wt = (over: Partial<Worktree> = {}): Worktree => ({
  path: '/r/wt', name: 'wt', branch: 'feat', oid: 'abcdef1234567890', isMain: false, isCurrent: false,
  missing: false, locked: false, ...over,
});
const ids = (w: Worktree) => worktreeMenu(w).map((i) => (i.disabled ? `(${i.id})` : i.id));

it('a linked worktree can be opened, shown, copied and removed', () => {
  const m = worktreeMenu(wt());
  expect(m.map((i) => i.label)).toEqual(['Open', 'Show in Finder', 'Copy path', 'Remove worktree']);
  const remove = m.find((i) => i.id === 'remove')!;
  expect(remove.danger).toBe(true);
  expect(remove.separatorBefore).toBe(true);
});

it('the main and the current worktree cannot be removed; the current one is already open', () => {
  expect(ids(wt({ isMain: true }))).toEqual(['open', 'reveal', 'copyPath', '(remove)']);
  expect(ids(wt({ isCurrent: true }))).toEqual(['(open)', 'reveal', 'copyPath', '(remove)']);
});

it('a missing worktree offers clean up instead', () => {
  const m = worktreeMenu(wt({ missing: true }));
  expect(ids(wt({ missing: true }))).toEqual(['(open)', '(reveal)', 'copyPath', 'prune']);
  expect(m.at(-1)!.label).toBe('Clean up');
});

it('labels a worktree by branch, else short commit', () => {
  expect(worktreeLabel(wt())).toBe('feat');
  expect(worktreeLabel(wt({ branch: null }))).toBe('abcdef1');
  expect(worktreeLabel(wt({ branch: null, oid: null }))).toBe('No commits');
});

it('maps branches checked out in other worktrees, not this one', () => {
  const m = checkedOutElsewhere([
    wt({ path: '/r', name: 'r', branch: 'main', isMain: true, isCurrent: true }),
    wt(),
    wt({ path: '/r/d', branch: null }),
  ]);
  expect([...m.keys()]).toEqual(['feat']);
  expect(m.get('feat')!.name).toBe('wt');
});

it('recognises git refusing to remove a worktree with changes', () => {
  expect(isDirtyWorktreeError({ kind: 'failed', message: 'x', stderr: "fatal: '/r/wt' contains modified or untracked files, use --force to delete it" })).toBe(true);
  expect(isDirtyWorktreeError({ kind: 'failed', message: 'x', stderr: 'fatal: cannot remove a locked working tree' })).toBe(false);
});

it('compares paths ignoring a trailing slash', () => {
  expect(samePath('/r/wt/', '/r/wt')).toBe(true);
  expect(samePath('/r/wt', '/r/wt2')).toBe(false);
});
