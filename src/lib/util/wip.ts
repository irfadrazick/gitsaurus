import type { Change, ChangedFile, FileStatus, Operation } from '../types';

/** A ChangedFile row for the working tree; `conflict` selects the conflict icon in FileList. */
export type WipFile = ChangedFile & { conflict?: boolean };

const CODE: Record<string, Change> = {
  A: 'added', M: 'modified', D: 'deleted', R: 'renamed', C: 'copied', T: 'typeChange', U: 'modified',
};

/** Map a porcelain entry to a list row for its index (staged) or worktree (unstaged) side. */
export function toWipFile(f: FileStatus, side: 'index' | 'worktree'): WipFile {
  const base = { path: f.path, origPath: f.origPath };
  if (f.conflicted) return { ...base, change: 'modified', conflict: true };
  if (f.untracked) return { ...base, change: 'added' };
  return { ...base, change: CODE[side === 'index' ? f.index : f.worktree] ?? 'modified' };
}

const NAME: Record<Operation['kind'], string> = {
  merge: 'Merge', rebase: 'Rebase', cherryPick: 'Cherry-pick', revert: 'Revert',
};

export const opName = (kind: Operation['kind']) => NAME[kind];

/**
 * Side names for the conflict resolve buttons ("Use <name>"). With no operation
 * in progress the conflict came from applying a stash: ours is the current
 * working tree, theirs the stash.
 */
export function conflictSides(op: Operation | null): { ours: string; theirs: string } {
  if (!op) return { ours: 'mine (current)', theirs: 'stash' };
  return { ours: `"${op.oursLabel}"`, theirs: `"${op.theirsLabel}"` };
}

/** Text of the conflict banner: what is happening (naming both sides when known), then the conflict count. */
export function bannerText(op: Operation, conflicted: number): string {
  const ours = op.oursLabel;
  const theirs = op.theirsLabel;
  let what: string;
  if (op.kind === 'merge') what = theirs && ours ? `Merging ${theirs} into ${ours}` : 'Merge';
  else if (op.kind === 'rebase') what = theirs && ours ? `Rebasing ${theirs} onto ${ours}` : 'Rebase';
  else if (op.kind === 'cherryPick') what = theirs ? `Cherry-picking ${theirs}` : 'Cherry-pick';
  else what = theirs ? `Reverting ${theirs}` : 'Revert';
  if (op.kind === 'rebase' && op.step !== null && op.total !== null) what += `, step ${op.step}/${op.total}`;
  const state =
    conflicted === 0 ? 'all conflicts resolved' : `${conflicted} conflicted ${conflicted === 1 ? 'file' : 'files'}`;
  return `${what}: ${state}`;
}

export const canCommit = (summary: string, staged: number, amend: boolean) =>
  summary.trim() !== '' && (staged > 0 || amend);

/** Summary fully trimmed; description loses leading blank lines (not indentation) and trailing whitespace. */
export function commitMessageParts(summary: string, description: string) {
  return { summary: summary.trim(), description: description.replace(/^(?:[ \t]*\n)+/, '').trimEnd() };
}
