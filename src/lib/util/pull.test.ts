import { expect, it } from 'vitest';
import type { Branch } from '../types';
import { pullPlan } from './pull';

const br = (over: Partial<Branch> = {}): Branch => ({
  name: 'feat/x', fullRef: 'refs/heads/feat/x', oid: 'a1', upstream: 'origin/feat/x',
  ahead: 0, behind: 0, gone: false, isHead: true, ...over,
});

it('pulls normally when the branch tracks an existing upstream', () => {
  expect(pullPlan(br(), ['origin/feat/x'], ['origin'])).toEqual({ kind: 'pull' });
});

it('pulls normally when the branch is unknown (detached HEAD: git reports it)', () => {
  expect(pullPlan(undefined, [], ['origin'])).toEqual({ kind: 'pull' });
});

it('offers to track a same-named branch on the remote, preferring origin', () => {
  const b = br({ upstream: null });
  expect(pullPlan(b, ['fork/feat/x', 'origin/feat/x'], ['fork', 'origin'])).toEqual({ kind: 'track', upstream: 'origin/feat/x' });
  expect(pullPlan(b, ['fork/feat/x'], ['fork', 'origin'])).toEqual({ kind: 'track', upstream: 'fork/feat/x' });
});

it('offers to publish a branch that is not on the remote', () => {
  expect(pullPlan(br({ upstream: null }), ['origin/main'], ['origin'])).toEqual({ kind: 'publish', remote: 'origin', gone: false });
});

it('offers to publish again when the upstream was deleted on the remote', () => {
  expect(pullPlan(br({ upstream: 'fork/feat/x', gone: true }), [], ['origin', 'fork'])).toEqual({ kind: 'publish', remote: 'fork', gone: true });
});

it('says so when there is no remote at all', () => {
  expect(pullPlan(br({ upstream: null }), [], [])).toEqual({ kind: 'noRemote' });
});
