import { expect, it } from 'vitest';
import { branchMenu, commitMenu, stashMenu, tagMenu, type MenuContext } from './menus';

const ctx = (over: Partial<MenuContext> = {}): MenuContext => ({
  name: 'feat', current: 'main', isHeadBranch: false, detached: false, hasUpstream: false, ...over,
});
const labels = (m: { label: string }[]) => m.map((i) => i.label);

it('branch menu for non-head branch', () => {
  const m = branchMenu(ctx());
  expect(labels(m)).toEqual([
    'Pull (fast-forward if possible)', 'Push', 'Set Upstream', 'Merge feat into main', 'Rebase main onto feat',
    'Checkout feat', 'Create branch here', 'Cherry pick commit', 'Reset main to this commit', 'Revert commit',
    'Rename feat', 'Delete feat', 'Copy branch name', 'Copy commit sha', 'Create tag here', 'Create annotated tag here',
  ]);
  const reset = m.find((i) => i.id === 'reset')!;
  expect(reset.submenu!.map((i) => i.label)).toEqual(['Soft', 'Mixed', 'Hard']);
  expect(reset.submenu!.map((i) => i.id)).toEqual(['reset.soft', 'reset.mixed', 'reset.hard']);
  expect(reset.submenu![2].danger).toBe(true);
  expect(m.find((i) => i.id === 'delete')!.danger).toBe(true);
});

it('head branch drops merge/rebase/checkout/delete', () => {
  const ids = branchMenu(ctx({ name: 'main', isHeadBranch: true })).map((i) => i.id);
  for (const id of ['merge', 'rebase', 'checkout', 'delete']) expect(ids).not.toContain(id);
  expect(ids).toContain('rename');
});

it('push disabled when detached', () => {
  const m = branchMenu(ctx({ name: 'main', isHeadBranch: true, detached: true, current: null }));
  expect(m.find((i) => i.id === 'push')!.disabled).toBe(true);
});

it('commit menu has no branch-specific items', () => {
  const ids = commitMenu(ctx()).map((i) => i.id);
  expect(ids).toEqual(expect.arrayContaining(['checkout', 'createBranch', 'cherryPick', 'revert', 'copySha', 'createTag']));
  expect(ids).not.toContain('merge');
});

it('tag and stash menus', () => {
  expect(labels(tagMenu(ctx({ name: 'v1' })))).toEqual(['Checkout v1', 'Push tag', 'Delete v1', 'Copy tag name']);
  expect(labels(stashMenu(ctx()))).toEqual(['Apply', 'Pop', 'Delete']);
  expect(stashMenu(ctx()).find((i) => i.id === 'stashDrop')!.danger).toBe(true);
});
