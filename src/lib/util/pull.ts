import type { Branch } from '../types';
import { remoteOfUpstream } from './actionTable';
import { defaultRemote } from './watchDiff';

/**
 * What Pull on the current branch should do, decided before running git so a branch
 * without a usable upstream gets a button instead of git's "no tracking information" text.
 */
export type PullPlan =
  | { kind: 'pull' }
  /** The remote has a branch of the same name: track it, then pull. */
  | { kind: 'track'; upstream: string }
  /** Nothing to pull: the branch isn't on the remote (or its upstream was deleted there). */
  | { kind: 'publish'; remote: string; gone: boolean }
  | { kind: 'noRemote' };

export function pullPlan(branch: Branch | undefined, remoteBranches: string[], remotes: string[]): PullPlan {
  if (!branch || (branch.upstream && !branch.gone)) return { kind: 'pull' };
  if (branch.upstream && branch.gone) {
    const remote = remoteOfUpstream(branch.upstream, remotes) ?? defaultRemote(remotes);
    return remote ? { kind: 'publish', remote, gone: true } : { kind: 'noRemote' };
  }
  const order = remotes.includes('origin') ? ['origin', ...remotes.filter((r) => r !== 'origin')] : remotes;
  const same = order.map((r) => `${r}/${branch.name}`).find((n) => remoteBranches.includes(n));
  if (same) return { kind: 'track', upstream: same };
  const remote = defaultRemote(remotes);
  return remote ? { kind: 'publish', remote, gone: false } : { kind: 'noRemote' };
}
