/** Compact age for a graph row, e.g. "now", "5m", "10h", "3d", "2w", "4mo", "1y". `ts` and `now` are unix seconds. */
export function shortAge(ts: number, now: number): string {
  const s = Math.max(0, now - ts);
  if (s < 60) return 'now';
  if (s < 3600) return `${Math.floor(s / 60)}m`;
  if (s < 86400) return `${Math.floor(s / 3600)}h`;
  const d = Math.floor(s / 86400);
  if (d < 7) return `${d}d`;
  if (d < 30) return `${Math.floor(d / 7)}w`;
  if (d < 365) return `${Math.floor(d / 30)}mo`;
  return `${Math.floor(d / 365)}y`;
}
