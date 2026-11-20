import { describe, expect, it } from 'vitest';
import type { RefLabel, Refs } from '../types';
import { graphRefsChanged, groupPills, pillsShown, refTree, rowPills, visibleRefs, type RefNode } from './refs';

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

describe('refTree', () => {
  const tree = (names: string[]) => refTree(names, (n) => n);
  // Folders as "name/[...]", refs as their last segment.
  const shape = (nodes: RefNode<string>[]): unknown[] =>
    nodes.map((n) => (n.kind === 'folder' ? { [`${n.name}/`]: shape(n.children) } : n.name));

  it('lists names without a slash as they are', () => {
    expect(shape(tree(['main', 'dev']))).toEqual(['main', 'dev']);
  });

  it('groups names by their slash-separated parts, in the order they come', () => {
    expect(shape(tree(['predev4', 'release/v4.6', 'preuat', 'release/v4.7', 'feat/a/b', 'feat/c']))).toEqual([
      'predev4', { 'release/': ['v4.6', 'v4.7'] }, 'preuat', { 'feat/': [{ 'a/': ['b'] }, 'c'] },
    ]);
  });

  it('keeps the item and gives folders their full path', () => {
    const [f] = refTree([{ n: 'release/v4.7' }], (x) => x.n);
    expect(f.kind === 'folder' && f.path).toBe('release');
    const leaf = f.kind === 'folder' ? f.children[0] : f;
    expect(leaf.kind === 'ref' && leaf.item).toEqual({ n: 'release/v4.7' });
  });
});

describe('visibleRefs', () => {
  it('lists refs in order and skips the contents of closed folders', () => {
    const t = refTree(['a', 'feat/x', 'feat/sub/y', 'b', 'fix/z'], (n) => n);
    expect(visibleRefs(t, () => true)).toEqual(['a', 'feat/x', 'feat/sub/y', 'b', 'fix/z']);
    expect(visibleRefs(t, (p) => p !== 'feat/sub')).toEqual(['a', 'feat/x', 'b', 'fix/z']);
    expect(visibleRefs(t, (p) => p !== 'feat')).toEqual(['a', 'b', 'fix/z']);
  });
});
