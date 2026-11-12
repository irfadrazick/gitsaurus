import type { MenuItem } from '../util/menus';

export interface OpenMenu { x: number; y: number; items: MenuItem[]; onPick: (id: string) => void }

/** The menu currently shown; ContextMenu.svelte (mounted once in App) renders it. */
export const contextMenu = $state<{ current: OpenMenu | null }>({ current: null });

/** Show a context menu at viewport coordinates. `onPick` receives the picked item id (leaf ids, e.g. `reset.hard`). */
export function openMenu(x: number, y: number, items: MenuItem[], onPick: (id: string) => void) {
  contextMenu.current = { x, y, items, onPick };
}

export function closeMenu() {
  contextMenu.current = null;
}
