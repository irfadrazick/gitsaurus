import { describe, expect, it } from 'vitest';
import { clickMarks, joinNames, NO_MARKS } from './marks';

const order = ['a', 'b', 'c', 'd', 'e'];
const plain = { meta: false, shift: false };
const cmd = { meta: true, shift: false };
const shift = { meta: false, shift: true };
const cmdShift = { meta: true, shift: true };

describe('clickMarks', () => {
  it('a plain click marks just that row and makes it the anchor', () => {
    expect(clickMarks({ names: ['a', 'b'], anchor: 'a' }, 'c', order, plain)).toEqual({ names: ['c'], anchor: 'c' });
  });
  it('⌘-click adds a row, or removes it when already marked', () => {
    const one = clickMarks(NO_MARKS, 'b', order, plain);
    const two = clickMarks(one, 'd', order, cmd);
    expect(two).toEqual({ names: ['b', 'd'], anchor: 'd' });
    expect(clickMarks(two, 'b', order, cmd)).toEqual({ names: ['d'], anchor: 'b' });
  });
  it('⇧-click marks the rows from the anchor to the clicked one, either way', () => {
    expect(clickMarks({ names: ['b'], anchor: 'b' }, 'd', order, shift)).toEqual({ names: ['b', 'c', 'd'], anchor: 'b' });
    expect(clickMarks({ names: ['d'], anchor: 'd' }, 'b', order, shift)).toEqual({ names: ['b', 'c', 'd'], anchor: 'd' });
  });
  it('a second ⇧-click replaces the range from the same anchor', () => {
    const r = clickMarks({ names: ['b'], anchor: 'b' }, 'e', order, shift);
    expect(clickMarks(r, 'c', order, shift)).toEqual({ names: ['b', 'c'], anchor: 'b' });
  });
  it('⌘⇧-click adds the range to what is already marked', () => {
    expect(clickMarks({ names: ['a', 'd'], anchor: 'd' }, 'e', order, cmdShift)).toEqual({ names: ['a', 'd', 'e'], anchor: 'd' });
  });
  it('⇧-click with no anchor in the list acts as a plain click', () => {
    expect(clickMarks(NO_MARKS, 'c', order, shift)).toEqual({ names: ['c'], anchor: 'c' });
    expect(clickMarks({ names: ['x'], anchor: 'x' }, 'c', order, shift)).toEqual({ names: ['c'], anchor: 'c' });
  });
});

describe('joinNames', () => {
  it('joins names for a sentence', () => {
    expect(joinNames([])).toBe('');
    expect(joinNames(['a'])).toBe('a');
    expect(joinNames(['a', 'b'])).toBe('a and b');
    expect(joinNames(['a', 'b', 'c'])).toBe('a, b and c');
  });
  it('lists the first few and counts the rest', () => {
    expect(joinNames(['a', 'b', 'c', 'd'], 3)).toBe('a, b, c and 1 more');
  });
});
