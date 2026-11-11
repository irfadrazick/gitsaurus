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

export function initials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (!parts.length) return '?';
  const first = parts[0][0];
  const last = parts.length > 1 ? parts[parts.length - 1][0] : '';
  return (first + last).toUpperCase();
}
