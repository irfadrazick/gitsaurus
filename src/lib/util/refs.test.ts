import { describe, expect, it } from 'vitest';
import type { RefLabel, Refs } from '../types';
import { graphRefsChanged, groupPills, pillTitle, pillsShown, rowPills } from './refs';

it('merges local and remote of the same branch', () => {
  const refs: RefLabel[] = [
    { kind: 'local', name: 'main', isHead: true },
    { kind: 'remote', name: 'origin/main', isHead: false },
    { kind: 'remote', name: 'origin/x', isHead: false },
    { kind: 'tag', name: 'v1', isHead: false },
  ];
  expect(groupPills(refs)).toEqual([
    { name: 'main', local: true, remote: true, tag: false, isHead: true },
    { name: 'origin/x', local: false, remote: true, tag: false, isHead: false },
    { name: 'v1', local: false, remote: false, tag: true, isHead: false },
  ]);
});

it('rowPills puts the checked-out branch first, keeping the rest in order', () => {
  const refs: RefLabel[] = [
    { kind: 'tag', name: 'v2', isHead: false },
    { kind: 'local', name: 'dev', isHead: false },
    { kind: 'local', name: 'main', isHead: true },
  ];
  expect(rowPills(refs).map((p) => p.name)).toEqual(['main', 'dev', 'v2']);
});

it('pillTitle describes every pill on its own line', () => {
  const pills = groupPills([
    { kind: 'local', name: 'main', isHead: true },
    { kind: 'remote', name: 'origin/main', isHead: false },
    { kind: 'local', name: 'dev', isHead: false },
    { kind: 'remote', name: 'origin/x', isHead: false },
    { kind: 'tag', name: 'v1', isHead: false },
  ]);
  expect(pillTitle(pills)).toBe('✓ main (local + remote)\ndev (local)\norigin/x (remote)\nv1 (tag)');
});

describe('pillsShown', () => {
  // Gap between pills is 4 px; the +N chip is 20 px wide below ten.
  const more = (n: number) => (n < 10 ? 20 : 26);
  it('shows up to two pills when they fit in full, with a +N chip for the rest', () => {
    expect(pillsShown([60, 50], more, 200)).toBe(2);
    expect(pillsShown([60, 50, 40], more, 200)).toBe(2);
    expect(pillsShown([100, 70, 40], more, 200)).toBe(2); // 100 + 4 + 70 + 4 + 20 = 198
  });
  it('drops to one pill and a count when two would not fit in full', () => {
    expect(pillsShown([120, 90], more, 200)).toBe(1); // 214
    expect(pillsShown([100, 80, 40], more, 200)).toBe(1); // 208
  });
  it('counts the gaps and the +N chip exactly', () => {
    expect(pillsShown([100, 76], more, 180)).toBe(2);
    expect(pillsShown([100, 76], more, 179)).toBe(1);
    expect(pillsShown([100, 50, 40], more, 178)).toBe(2);
    expect(pillsShown([100, 50, 40], more, 177)).toBe(1);
  });
  it('always shows one pill, and the most when the room is not known yet', () => {
    expect(pillsShown([500], more, 100)).toBe(1);
    expect(pillsShown([500, 500], more, 100)).toBe(1);
    expect(pillsShown([], more, 100)).toBe(0);
    expect(pillsShown([300, 300, 300], more, 0)).toBe(2);
  });
});

describe('graphRefsChanged', () => {
  const refs = (over: Partial<Refs> = {}): Refs => ({
    head: { branch: 'main', oid: 'a1' },
    local: [{ name: 'main', fullRef: 'refs/heads/main', oid: 'a1', upstream: 'origin/main', ahead: 0, behind: 0, gone: false, isHead: true }],
    remote: [{ remote: 'origin', name: 'main', fullName: 'origin/main', oid: 'a1' }],
    remotes: ['origin'],
    tags: [{ name: 'v1', oid: 'a0', annotated: false }],
    stashes: [],
    worktrees: [],
    ...over,
  });
  it('is false when the branches, remote branches, tags and HEAD are the same', () => {
    expect(graphRefsChanged(refs(), refs())).toBe(false);
  });
  it('ignores what the graph does not draw: stashes and worktrees', () => {
    expect(graphRefsChanged(refs(), refs({ stashes: [{ index: 0, message: 'wip', oid: 's1' }] }))).toBe(false);
  });
  it('is true when a branch moves, appears or is renamed', () => {
    const moved = refs();
    moved.local[0].oid = 'b2';
    expect(graphRefsChanged(refs(), moved)).toBe(true);
    const renamed = refs();
    renamed.local[0].name = 'trunk';
    expect(graphRefsChanged(refs(), renamed)).toBe(true);
  });
  it('is true when a remote branch, a tag or HEAD changes', () => {
    expect(graphRefsChanged(refs(), refs({ remote: [] }))).toBe(true);
    expect(graphRefsChanged(refs(), refs({ tags: [{ name: 'v1', oid: 'a1', annotated: false }] }))).toBe(true);
    expect(graphRefsChanged(refs(), refs({ head: { branch: null, oid: 'a1' } }))).toBe(true);
    expect(graphRefsChanged(refs(), refs({ head: { branch: 'main', oid: 'a0' } }))).toBe(true);
  });
});
