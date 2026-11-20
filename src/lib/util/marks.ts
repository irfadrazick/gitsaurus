/** Rows marked in a list (by name), and the row a ⇧-click range starts from. */
export interface Marks {
  names: string[];
  anchor: string | null;
}

export const NO_MARKS: Marks = { names: [], anchor: null };

/**
 * Marks after clicking `name`, as in a Finder list: a plain click marks just that row, ⌘-click adds
 * or removes it, and ⇧-click marks every row from the anchor to it (⌘⇧ adds that range to the
 * marks). `order` is the rows as listed; the range is taken from it.
 */
export function clickMarks(m: Marks, name: string, order: string[], mod: { meta: boolean; shift: boolean }): Marks {
  const from = m.anchor === null ? -1 : order.indexOf(m.anchor);
  const to = order.indexOf(name);
  if (mod.shift && from >= 0 && to >= 0) {
    const range = order.slice(Math.min(from, to), Math.max(from, to) + 1);
    const names = mod.meta ? [...m.names, ...range.filter((n) => !m.names.includes(n))] : range;
    return { names, anchor: m.anchor };
  }
  if (mod.meta) {
    const names = m.names.includes(name) ? m.names.filter((n) => n !== name) : [...m.names, name];
    return { names, anchor: name };
  }
  return { names: [name], anchor: name };
}

/** Names for a sentence: "a, b and c", or the first `max` and "and N more". */
export function joinNames(names: string[], max = 5): string {
  if (names.length <= 1) return names.join('');
  if (names.length > max) return `${names.slice(0, max).join(', ')} and ${names.length - max} more`;
  return `${names.slice(0, -1).join(', ')} and ${names[names.length - 1]}`;
}
