/** A commit timestamp (unix seconds) as a medium date and short time in the user's locale, e.g. "6 Oct 2026, 14:00". */
export function formatDate(ts: number, locale?: string): string {
  return new Intl.DateTimeFormat(locale, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(ts * 1000));
}
