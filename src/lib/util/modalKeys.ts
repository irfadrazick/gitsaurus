/**
 * Whether a global Enter keypress should confirm the open dialog.
 * Danger confirms never bind Enter; otherwise a focused button handles Enter itself.
 */
export function shouldEnterConfirm(kind: 'confirm' | 'prompt' | 'choose', danger: boolean, activeTag: string | null): boolean {
  if (kind !== 'confirm' || danger) return false;
  return activeTag?.toLowerCase() !== 'button';
}
