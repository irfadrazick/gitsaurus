import { SvelteMap, SvelteSet } from 'svelte/reactivity';
import * as api from '../api';
import type { GitError } from '../types';

/** Per-open-repo lightweight status: changed-file count (dirty dot) and "missing" (no longer a repo). */
class RepoHealth {
  /** repo path -> number of changed files. */
  dirty = new SvelteMap<string, number>();
  missing = new SvelteSet<string>();

  /** Re-validate `path` and refresh its file count. */
  async check(path: string) {
    try {
      await api.openRepo(path);
    } catch (e) {
      if ((e as Partial<GitError>)?.kind === 'notARepo') {
        this.missing.add(path);
        this.dirty.delete(path);
      }
      return;
    }
    this.missing.delete(path);
    try {
      this.dirty.set(path, (await api.repoState(path)).files.length);
    } catch {
      /* keep the previous count */
    }
  }

  checkAll(paths: string[]) {
    return Promise.all(paths.map((p) => this.check(p)));
  }

  /** Cheap update when the caller already has the count (the selected repo's loaded state). */
  setCount(path: string, n: number) {
    this.dirty.set(path, n);
  }

  forget(path: string) {
    this.dirty.delete(path);
    this.missing.delete(path);
  }
}

export const health = new RepoHealth();
