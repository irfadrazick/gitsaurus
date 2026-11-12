import type { FileStatus } from '../types';

/**
 * True when a porcelain status code means "changed" on that side. The backend
 * (porcelain v2) uses '.' for unchanged and '?' for untracked; ' ' is v1's
 * spelling of unchanged and is tolerated too.
 */
export const changed = (code: string) => code !== '.' && code !== ' ' && code !== '?';

/** Partition porcelain status: conflicted files appear only in `conflicted`; 'MM' appears in both lists. */
export function splitStatus(files: FileStatus[]) {
  const unstaged: FileStatus[] = [];
  const staged: FileStatus[] = [];
  const conflicted: FileStatus[] = [];
  for (const f of files) {
    if (f.conflicted) conflicted.push(f);
    else if (f.untracked) unstaged.push(f);
    else {
      if (changed(f.index)) staged.push(f);
      if (changed(f.worktree)) unstaged.push(f);
    }
  }
  return { unstaged, staged, conflicted };
}
