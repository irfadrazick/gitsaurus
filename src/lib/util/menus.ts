export interface MenuItem {
  /** Stable id; Task 13's performMenuAction switches on it. */
  id: string;
  label: string;
  danger?: boolean;
  disabled?: boolean;
  separatorBefore?: boolean;
  submenu?: MenuItem[];
  /** Right-aligned secondary text, e.g. a shortcut or "Default". */
  hint?: string;
}

/**
 * Context for building a context menu.
 * - name: the branch / tag name the menu was opened on (unused by commitMenu/stashMenu)
 * - current: the checked-out branch, or null when HEAD is detached
 * - isHeadBranch: `name` is the checked-out branch
 * - detached: HEAD is detached (Push is disabled)
 * - hasUpstream: the branch has an upstream configured
 */
export interface MenuContext {
  name: string;
  current: string | null;
  isHeadBranch: boolean;
  detached: boolean;
  hasUpstream: boolean;
}

const resetItem = (current: string | null): MenuItem => ({
  id: 'reset',
  label: `Reset ${current ?? 'HEAD'} to this commit`,
  separatorBefore: true,
  submenu: [
    { id: 'reset.soft', label: 'Soft: keep changes staged' },
    { id: 'reset.mixed', label: 'Mixed: keep changes unstaged' },
    { id: 'reset.hard', label: 'Hard: discard all changes', danger: true },
  ],
});

const tagItems = (): MenuItem[] => [
  { id: 'createTag', label: 'Create tag here', separatorBefore: true },
  { id: 'createAnnotatedTag', label: 'Create annotated tag here' },
];

export function branchMenu(c: MenuContext): MenuItem[] {
  const cur = c.current ?? 'HEAD';
  const items: MenuItem[] = [
    { id: 'pull', label: 'Pull (fast-forward if possible)', disabled: c.detached || !c.hasUpstream },
    { id: 'push', label: 'Push', disabled: c.detached },
    { id: 'setUpstream', label: 'Set upstream', disabled: c.detached },
  ];
  if (!c.isHeadBranch) {
    items.push(
      { id: 'merge', label: `Merge ${c.name} into ${cur}`, separatorBefore: true, disabled: c.detached },
      { id: 'rebase', label: `Rebase ${cur} onto ${c.name}`, disabled: c.detached },
      { id: 'checkout', label: `Checkout ${c.name}` },
    );
  }
  items.push(
    { id: 'createBranch', label: 'Create branch here', separatorBefore: c.isHeadBranch },
    { id: 'cherryPick', label: 'Cherry-pick commit' },
    resetItem(c.current),
    { id: 'revert', label: 'Revert commit' },
    { id: 'rename', label: `Rename ${c.name}`, separatorBefore: true },
  );
  if (!c.isHeadBranch) items.push({ id: 'delete', label: `Delete ${c.name}`, danger: true });
  items.push(
    { id: 'copyBranchName', label: 'Copy branch name', separatorBefore: true },
    { id: 'copySha', label: 'Copy commit SHA' },
    ...tagItems(),
  );
  return items;
}

/** Menu for a commit row that has no branch under the cursor. */
export function commitMenu(c: MenuContext): MenuItem[] {
  return [
    { id: 'checkout', label: 'Checkout this commit' },
    { id: 'createBranch', label: 'Create branch here' },
    { id: 'cherryPick', label: 'Cherry-pick commit', separatorBefore: true },
    resetItem(c.current),
    { id: 'revert', label: 'Revert commit' },
    { id: 'copySha', label: 'Copy commit SHA', separatorBefore: true },
    ...tagItems(),
  ];
}

export function tagMenu(c: MenuContext): MenuItem[] {
  const cur = c.current ?? 'HEAD';
  return [
    { id: 'merge', label: `Merge ${c.name} into ${cur}`, disabled: c.detached },
    { id: 'rebase', label: `Rebase ${cur} onto ${c.name}`, disabled: c.detached },
    { id: 'checkout', label: `Checkout ${c.name}` },
    { id: 'createBranch', label: 'Create branch here', separatorBefore: true },
    { id: 'cherryPick', label: 'Cherry-pick commit' },
    resetItem(c.current),
    { id: 'revert', label: 'Revert commit' },
    { id: 'pushTag', label: 'Push tag', separatorBefore: true },
    { id: 'copyTagName', label: 'Copy tag name', separatorBefore: true },
    { id: 'copySha', label: 'Copy commit SHA' },
    { id: 'delete', label: `Delete ${c.name}`, danger: true, separatorBefore: true },
  ];
}

export function stashMenu(_c: MenuContext): MenuItem[] {
  return [
    { id: 'stashApply', label: 'Apply' },
    { id: 'stashPop', label: 'Pop' },
    { id: 'stashDrop', label: 'Delete', danger: true, separatorBefore: true },
  ];
}

/** Menu for a remote-tracking branch row. */
export function remoteBranchMenu(c: MenuContext): MenuItem[] {
  return [
    { id: 'checkout', label: `Checkout ${c.name}` },
    { id: 'copyBranchName', label: 'Copy branch name', separatorBefore: true },
    { id: 'delete', label: 'Delete on remote', danger: true, separatorBefore: true },
  ];
}

/** Menu for a row in the Worktrees list. */
export function worktreeMenu(w: { isMain: boolean; isCurrent: boolean; missing: boolean }): MenuItem[] {
  return [
    { id: 'open', label: 'Open', disabled: w.isCurrent || w.missing },
    { id: 'reveal', label: 'Show in Finder', disabled: w.missing },
    { id: 'copyPath', label: 'Copy path' },
    w.missing
      ? { id: 'prune', label: 'Clean up', separatorBefore: true }
      : { id: 'remove', label: 'Remove worktree', danger: true, separatorBefore: true, disabled: w.isMain || w.isCurrent },
  ];
}

/**
 * Menu for several marked branch rows. `deletable` leaves out the checked-out branch, which
 * can't be deleted; remote rows are deleted on their remote.
 */
export function marksMenu(kind: 'local' | 'remote', deletable: number): MenuItem[] {
  const n = `${deletable} ${deletable === 1 ? 'branch' : 'branches'}`;
  return [
    { id: 'copyNames', label: 'Copy branch names' },
    kind === 'local'
      ? { id: 'deleteMarked', label: `Delete ${n}`, danger: true, separatorBefore: true, disabled: deletable === 0 }
      : { id: 'deleteMarked', label: `Delete ${n} on remote`, danger: true, separatorBefore: true },
  ];
}
