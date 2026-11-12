import { expect, it } from 'vitest';
import type { RefLabel } from '../types';
import { groupPills, pillTitle, rowPills } from './refs';

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
