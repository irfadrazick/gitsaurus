import type { Centre } from '../state/repo.svelte';
import type { DiffSource, Operation } from '../types';

/** Diffs of the working state (not of a commit) go stale whenever the repo state refreshes. */
export const isLiveSource = (s: DiffSource) => s.kind !== 'commit';

/** The centre shows a conflicted file (workingFile) and the operation that caused it has just ended. */
export function leavesConflictView(prev: Operation | null, next: Operation | null, centre: Centre): boolean {
  return prev !== null && next === null && centre.kind === 'diff' && centre.source.kind === 'workingFile';
}
