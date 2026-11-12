/** Index of the hunk header to scroll to when stepping from `scrollTop` (`tops` ascending); null at the ends. */
export function stepHunk(tops: number[], scrollTop: number, dir: 1 | -1): number | null {
  if (dir === 1) {
    const i = tops.findIndex((t) => t > scrollTop + 1);
    return i < 0 ? null : i;
  }
  for (let i = tops.length - 1; i >= 0; i--) if (tops[i] < scrollTop - 1) return i;
  return null;
}
