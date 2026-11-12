import { describe, expect, it } from 'vitest';
import { shouldCloseDiffOnEsc } from './diffKeys';

describe('shouldCloseDiffOnEsc', () => {
  it('closes on a plain Escape', () => {
    expect(shouldCloseDiffOnEsc({ key: 'Escape', defaultPrevented: false })).toBe(true);
  });
  it('ignores Escape already consumed by a modal, menu or search', () => {
    expect(shouldCloseDiffOnEsc({ key: 'Escape', defaultPrevented: true })).toBe(false);
  });
  it('ignores other keys', () => {
    expect(shouldCloseDiffOnEsc({ key: 'a', defaultPrevented: false })).toBe(false);
  });
});
