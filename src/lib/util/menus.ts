export interface MenuItem {
  /** Stable id; Task 13's performMenuAction switches on it. */
  id: string;
  label: string;
  danger?: boolean;
  disabled?: boolean;
  separatorBefore?: boolean;
  submenu?: MenuItem[];
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
    { id: 'reset.soft', label: 'Soft' },
    { id: 'reset.mixed', label: 'Mixed' },
    { id: 'reset.hard', label: 'Hard', danger: true },
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
    { id: 'setUpstream', label: 'Set Upstream', disabled: c.detached },
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
    { id: 'cherryPick', label: 'Cherry pick commit' },
    resetItem(c.current),
    { id: 'revert', label: 'Revert commit' },
    { id: 'rename', label: `Rename ${c.name}`, separatorBefore: true },
  );
  if (!c.isHeadBranch) items.push({ id: 'delete', label: `Delete ${c.name}`, danger: true });
  items.push(
    { id: 'copyBranchName', label: 'Copy branch name', separatorBefore: true },
    { id: 'copySha', label: 'Copy commit sha' },
    ...tagItems(),
  );
  return items;
}

/** Menu for a commit row that has no branch under the cursor. */
export function commitMenu(c: MenuContext): MenuItem[] {
  return [
    { id: 'checkout', label: 'Checkout this commit' },
    { id: 'createBranch', label: 'Create branch here' },
    { id: 'cherryPick', label: 'Cherry pick commit', separatorBefore: true },
    resetItem(c.current),
    { id: 'revert', label: 'Revert commit' },
    { id: 'copySha', label: 'Copy commit sha', separatorBefore: true },
    ...tagItems(),
  ];
}

export function tagMenu(c: MenuContext): MenuItem[] {
  return [
    { id: 'checkout', label: `Checkout ${c.name}` },
    { id: 'pushTag', label: 'Push tag' },
    { id: 'delete', label: `Delete ${c.name}`, danger: true },
    { id: 'copyTagName', label: 'Copy tag name', separatorBefore: true },
  ];
}

export function stashMenu(_c: MenuContext): MenuItem[] {
  return [
    { id: 'stashApply', label: 'Apply' },
    { id: 'stashPop', label: 'Pop' },
    { id: 'stashDrop', label: 'Delete', danger: true },
  ];
}

/** Menu for a remote-tracking branch row. */
export function remoteBranchMenu(c: MenuContext): MenuItem[] {
  return [
    { id: 'checkout', label: `Checkout ${c.name}` },
    { id: 'copyBranchName', label: 'Copy branch name', separatorBefore: true },
  ];
}
