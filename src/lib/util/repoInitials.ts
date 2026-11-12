/** Last path segment of a repository path. */
export function repoName(path: string): string {
  const parts = path.split('/').filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

/** Two-letter badge: first letters of the first two `-`/`_`/`.` separated words, else the first two letters. */
export function repoInitials(name: string): string {
  const words = name.split(/[-_.]+/).filter(Boolean);
  const s = words.length >= 2 ? words[0][0] + words[1][0] : (words[0] ?? '').slice(0, 2);
  return s ? s.toUpperCase() : '?';
}
