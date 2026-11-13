import { palette } from './graphGeometry';

async function sha256Hex(s: string): Promise<string> {
  const buf = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(s));
  return [...new Uint8Array(buf)].map((b) => b.toString(16).padStart(2, '0')).join('');
}

/** Avatar image URL for an author email (GitHub noreply addresses, else Gravatar; d=404 lets the UI fall back to initials). */
export async function avatarUrl(email: string): Promise<string> {
  const e = email.trim().toLowerCase();
  const gh = /^(?:(\d+)\+)?([^@+]+)@users\.noreply\.github\.com$/.exec(e);
  if (gh) {
    return gh[1]
      ? `https://avatars.githubusercontent.com/u/${gh[1]}?s=64&v=4`
      : `https://github.com/${gh[2]}.png?size=64`;
  }
  return `https://gravatar.com/avatar/${await sha256Hex(e)}?s=64&d=404`;
}

/** Up to two initials, from the first and last words that start with a letter ("Claude Opus 5.5" → "CO"). */
export function initials(name: string): string {
  const all = name.trim().split(/\s+/).filter(Boolean);
  const words = all.filter((w) => /^\p{L}/u.test(w));
  const parts = words.length ? words : all;
  if (!parts.length) return '?';
  const first = [...parts[0]][0];
  const last = parts.length > 1 ? [...parts[parts.length - 1]][0] : '';
  return (first + last).toUpperCase();
}

/** Lane colours minus the red, which reads as an error behind an initials avatar. */
const PERSON_COLORS = palette.filter((c) => c !== '#f85149');

/** A stable fill for a person's initials avatar, picked from their email. */
export function personColor(email: string): string {
  let h = 0;
  for (const c of email) h = (h * 31 + c.charCodeAt(0)) >>> 0;
  return PERSON_COLORS[h % PERSON_COLORS.length];
}

/** Network avatar lookups wait for the saved settings, so "avatars off" never leaks a request at startup. */
export const avatarsAllowed = (s: { loaded: boolean; avatarsEnabled: boolean }) => s.loaded && s.avatarsEnabled;
