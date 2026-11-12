/** Esc closes the diff only if no other layer (modal, menu, search, settings) already consumed it. */
export function shouldCloseDiffOnEsc(e: { key: string; defaultPrevented: boolean }): boolean {
  return e.key === 'Escape' && !e.defaultPrevented;
}
