const p2 = (n: number) => String(n).padStart(2, '0');

/** `MM/DD/YYYY @ h:mm A` in local time. `ts` is unix seconds. */
export function formatDate(ts: number): string {
  const d = new Date(ts * 1000);
  const h = d.getHours();
  const h12 = h % 12 === 0 ? 12 : h % 12;
  return `${p2(d.getMonth() + 1)}/${p2(d.getDate())}/${d.getFullYear()} @ ${h12}:${p2(d.getMinutes())} ${h < 12 ? 'AM' : 'PM'}`;
}
