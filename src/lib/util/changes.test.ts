import { describe, expect, it } from 'vitest';
import type { ChangedFile } from '../types';
import { changeSummary } from './changes';

const f = (change: ChangedFile['change']): ChangedFile => ({ path: 'a', origPath: null, change });

describe('changeSummary', () => {
  it('counts per kind in a fixed order, skipping zeros', () => {
    const files = [f('added'), f('modified'), f('added'), f('modified'), f('modified'), f('added'), f('added')];
    expect(changeSummary(files)).toBe('3 modified · 4 added');
  });
  it('covers deleted, renamed and other kinds', () => {
    expect(changeSummary([f('deleted'), f('renamed'), f('copied'), f('typeChange')])).toBe(
      '1 deleted · 1 renamed · 1 copied · 1 type changed',
    );
  });
  it('is empty for no files', () => {
    expect(changeSummary([])).toBe('');
  });
});
