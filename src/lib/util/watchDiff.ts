/** Which repos to start / stop watching when the open list changes. */
export function watchDiff(prev: Iterable<string>, next: Iterable<string>): { add: string[]; remove: string[] } {
  const p = new Set(prev);
  const n = new Set(next);
  return { add: [...n].filter((x) => !p.has(x)), remove: [...p].filter((x) => !n.has(x)) };
}

/** Remote to offer for deleting a tag: origin when present, else the only/first remote. */
export function defaultRemote(remotes: string[]): string | null {
  return remotes.includes('origin') ? 'origin' : (remotes[0] ?? null);
}
