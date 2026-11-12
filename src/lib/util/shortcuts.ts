export type ShortcutAction =
  | 'toggleRepos' | 'search' | 'openRepo' | 'closeRepo' | 'settings' | 'escape' | { repo: number };

export interface KeyLike { key: string; metaKey: boolean; ctrlKey: boolean; altKey: boolean; shiftKey: boolean }

const CMD: Record<string, ShortcutAction> = {
  b: 'toggleRepos', f: 'search', o: 'openRepo', w: 'closeRepo', ',': 'settings',
};

export function matchShortcut(e: KeyLike): ShortcutAction | null {
  if (e.key === 'Escape') return 'escape';
  if (!e.metaKey || e.ctrlKey || e.altKey) return null;
  if (/^[1-9]$/.test(e.key)) return { repo: Number(e.key) };
  return CMD[e.key.toLowerCase()] ?? null;
}
