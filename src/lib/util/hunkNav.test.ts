import { describe, expect, it } from 'vitest';
import { stepHunk } from './hunkNav';

const tops = [0, 100, 300, 600];

describe('stepHunk', () => {
  it('next goes to the first header below the scroll position', () => {
    expect(stepHunk(tops, 0, 1)).toBe(1);
    expect(stepHunk(tops, 150, 1)).toBe(2);
  });
  it('prev goes to the last header above the scroll position', () => {
    expect(stepHunk(tops, 150, -1)).toBe(1);
    expect(stepHunk(tops, 100, -1)).toBe(0);
  });
  it('returns null at the ends', () => {
    expect(stepHunk(tops, 600, 1)).toBeNull();
    expect(stepHunk(tops, 0, -1)).toBeNull();
    expect(stepHunk([], 0, 1)).toBeNull();
  });
  it('ignores sub-pixel differences', () => {
    expect(stepHunk(tops, 99.5, 1)).toBe(2);
  });
});
