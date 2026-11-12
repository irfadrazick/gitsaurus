export type GitAction = 'pull' | 'push' | 'fetch' | 'newBranch' | 'stageAll' | 'unstageAll';
export type ShortcutAction =
  | 'toggleRepos' | 'search' | 'openRepo' | 'closeRepo' | 'settings' | 'escape' | GitAction | { repo: number };

const GIT_ACTIONS: readonly string[] = ['pull', 'push', 'fetch', 'newBranch', 'stageAll', 'unstageAll'];
export const isGitAction = (a: ShortcutAction): a is GitAction => typeof a === 'string' && GIT_ACTIONS.includes(a);

export interface KeyLike { key: string; metaKey: boolean; ctrlKey: boolean; altKey: boolean; shiftKey: boolean }

/** ⌘ + key. */
const CMD: Record<string, ShortcutAction> = {
  b: 'toggleRepos', f: 'search', o: 'openRepo', w: 'closeRepo', ',': 'settings', p: 'push',
};

/** ⌘⇧ + key. */
const CMD_SHIFT: Record<string, ShortcutAction> = {
  p: 'pull', f: 'fetch', n: 'newBranch', s: 'stageAll', u: 'unstageAll',
};

/** Shortcut list for the Settings reference, in display order. */
export const SHORTCUTS: { keys: string; label: string }[] = [
  { keys: '⌘⇧P', label: 'Pull' },
  { keys: '⌘P', label: 'Push' },
  { keys: '⌘⇧F', label: 'Fetch' },
  { keys: '⌘⇧N', label: 'New branch' },
  { keys: '⌘⇧S', label: 'Stage all' },
  { keys: '⌘⇧U', label: 'Unstage all' },
  { keys: 'Space', label: 'Stage or unstage the selected file' },
  { keys: '↑ ↓', label: 'Move through commits or changed files' },
  { keys: '⌘↩', label: 'Commit' },
  { keys: '⌘F', label: 'Search commits' },
  { keys: '[ ]', label: 'Previous / next change in a diff' },
  { keys: '⇧F10', label: 'Open the menu for the focused item' },
  { keys: '⌘B', label: 'Show or hide repositories' },
  { keys: '⌘1–9', label: 'Switch repository' },
  { keys: '⌘O', label: 'Open repository' },
  { keys: '⌘,', label: 'Settings' },
];

export function matchShortcut(e: KeyLike): ShortcutAction | null {
  if (e.key === 'Escape') return 'escape';
  if (!e.metaKey || e.ctrlKey || e.altKey) return null;
  const key = e.key.toLowerCase();
  if (e.shiftKey) return CMD_SHIFT[key] ?? null;
  if (/^[1-9]$/.test(e.key)) return { repo: Number(e.key) };
  return CMD[key] ?? null;
}
