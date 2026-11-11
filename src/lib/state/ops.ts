import { SvelteMap } from 'svelte/reactivity';
import type { GitError } from '../types';
import { repo } from './repo.svelte';
import { toasts } from './toasts.svelte';

/** In-flight operations by label (reactive). */
export const busy = new SvelteMap<string, boolean>();

export interface RunOpts { onPushRejected?: () => void }

/**
 * Run a backend operation with busy tracking, toasts and a post-success refresh.
 * Returns undefined when it failed. An OpOutcome of "conflicts" shows no toast (the banner takes over).
 */
export async function runOp<T>(label: string, fn: () => Promise<T>, opts: RunOpts = {}): Promise<T | undefined> {
  busy.set(label, true);
  try {
    const result = await fn();
    if (result !== 'conflicts') toasts.push({ kind: 'ok', title: `${label} done` });
    await repo.refresh(['Git', 'Worktree']);
    return result;
  } catch (e) {
    const err = e as Partial<GitError>;
    if (err?.kind === 'pushRejected' && opts.onPushRejected) {
      opts.onPushRejected();
    } else {
      const stderr = (err as { stderr?: string } | null)?.stderr;
      toasts.push({ kind: 'error', title: `${label} failed`, detail: stderr || err?.message || String(e) });
    }
    return undefined;
  } finally {
    busy.delete(label);
  }
}
