import type { FileStatus } from '../types';

/** Partition porcelain status: conflicted files appear only in `conflicted`; 'MM' appears in both lists. */
export function splitStatus(files: FileStatus[]) {
  const unstaged: FileStatus[] = [];
  const staged: FileStatus[] = [];
  const conflicted: FileStatus[] = [];
  for (const f of files) {
    if (f.conflicted) conflicted.push(f);
    else if (f.untracked) unstaged.push(f);
    else {
      if (f.index !== ' ' && f.index !== '?') staged.push(f);
      if (f.worktree !== ' ' && f.worktree !== '?') unstaged.push(f);
    }
  }
  return { unstaged, staged, conflicted };
}
