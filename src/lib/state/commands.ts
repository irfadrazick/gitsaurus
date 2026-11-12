/** Commands a keyboard shortcut asks a mounted component to run (the toolbar owns pull/push/fetch/new branch). */
export type Command = 'pull' | 'push' | 'fetch' | 'newBranch';

const bus = new EventTarget();

export function runCommand(c: Command) {
  bus.dispatchEvent(new Event(c));
}

/** Subscribe to a command; returns the unsubscribe function (usable as an $effect teardown). */
export function onCommand(c: Command, fn: () => void): () => void {
  bus.addEventListener(c, fn);
  return () => bus.removeEventListener(c, fn);
}
