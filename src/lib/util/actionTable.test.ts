import { expect, it } from 'vitest';
import { actionFor, defaultUpstream, isNotMerged, remoteOfUpstream } from './actionTable';
import { branchMenu, commitMenu, remoteBranchMenu, stashMenu, tagMenu, type MenuContext, type MenuItem } from './menus';

const ctx = (over: Partial<MenuContext> = {}): MenuContext => ({
  name: 'feat', current: 'main', isHeadBranch: false, detached: false, hasUpstream: true, ...over,
});
const leaves = (items: MenuItem[]): string[] => items.flatMap((i) => (i.submenu ? leaves(i.submenu) : [i.id]));

it('every id of every menu has a handler', () => {
  const cases = [
    ['branch', branchMenu(ctx())],
    ['branch', branchMenu(ctx({ isHeadBranch: true }))],
    ['commit', commitMenu(ctx())],
    ['tag', tagMenu(ctx())],
    ['stash', stashMenu(ctx())],
    ['remoteBranch', remoteBranchMenu(ctx())],
  ] as const;
  for (const [kind, items] of cases) for (const id of leaves(items)) expect(actionFor(kind, id), `${kind}:${id}`).not.toBeNull();
});

it('same id means different actions per menu kind', () => {
  expect(actionFor('branch', 'checkout')).toBe('checkoutBranch');
  expect(actionFor('commit', 'checkout')).toBe('checkoutCommit');
  expect(actionFor('tag', 'checkout')).toBe('checkoutTag');
  expect(actionFor('remoteBranch', 'checkout')).toBe('checkoutRemote');
  expect(actionFor('branch', 'delete')).toBe('deleteBranch');
  expect(actionFor('tag', 'delete')).toBe('deleteTag');
  expect(actionFor('remoteBranch', 'delete')).toBe('deleteRemoteBranch');
  expect(actionFor('commit', 'nope')).toBeNull();
});

it('defaultUpstream prefers origin/<name>, else null', () => {
  expect(defaultUpstream('feat', ['upstream/feat', 'origin/feat'])).toBe('origin/feat');
  expect(defaultUpstream('feat', ['upstream/feat'])).toBeNull();
  expect(defaultUpstream('feat', [])).toBeNull();
});

it('remoteOfUpstream finds the longest matching remote', () => {
  expect(remoteOfUpstream('origin/feat/x', ['origin'])).toBe('origin');
  expect(remoteOfUpstream('a/b/main', ['a', 'a/b'])).toBe('a/b');
  expect(remoteOfUpstream('zzz/main', ['origin'])).toBeNull();
});

it('isNotMerged detects the git message on failed errors only', () => {
  expect(isNotMerged({ kind: 'failed', message: 'x', stderr: "error: the branch 'x' is not fully merged." })).toBe(true);
  expect(isNotMerged({ kind: 'failed', message: 'x', stderr: 'other' })).toBe(false);
  expect(isNotMerged({ kind: 'pushRejected', message: 'x', stderr: 'not fully merged' })).toBe(false);
  expect(isNotMerged(null)).toBe(false);
});
