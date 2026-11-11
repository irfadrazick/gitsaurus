import { expect, it } from 'vitest';
import type { RefLabel } from '../types';
import { groupPills } from './refs';

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
