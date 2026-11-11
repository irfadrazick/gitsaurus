const plural = (n: number, unit: string, one: string) => (n === 1 ? one : `${n} ${unit}s ago`);

/** Coarse relative-time label for graph time markers. `ts` and `now` are unix seconds. */
export function timeBucket(ts: number, now: number): string {
  const s = Math.max(0, now - ts);
  if (s < 60) return 'just now';
  if (s < 3600) return plural(Math.floor(s / 60), 'minute', 'a minute ago');
  if (s < 86400) return plural(Math.floor(s / 3600), 'hour', '1 hour ago');
  const d = Math.floor(s / 86400);
  if (d < 7) return plural(d, 'day', 'yesterday');
  if (d < 30) return plural(Math.floor(d / 7), 'week', 'a week ago');
  if (d < 365) return plural(Math.floor(d / 30), 'month', 'a month ago');
  return plural(Math.floor(d / 365), 'year', 'a year ago');
}

/** True when `cur` falls in a different bucket than `prev` (or there is no previous row). */
export function showMarker(prev: number | undefined, cur: number, now: number): boolean {
  return prev === undefined || timeBucket(prev, now) !== timeBucket(cur, now);
}
