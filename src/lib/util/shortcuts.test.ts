import { expect, it } from 'vitest';
import { matchShortcut } from './shortcuts';

const k = (key: string, meta = true) => ({ key, metaKey: meta, ctrlKey: false, altKey: false, shiftKey: false });

it('maps keys', () => {
  expect(matchShortcut(k('b'))).toBe('toggleRepos');
  expect(matchShortcut(k('f'))).toBe('search');
  expect(matchShortcut(k('o'))).toBe('openRepo');
  expect(matchShortcut(k('w'))).toBe('closeRepo');
  expect(matchShortcut(k(','))).toBe('settings');
  expect(matchShortcut(k('3'))).toEqual({ repo: 3 });
  expect(matchShortcut(k('Escape', false))).toBe('escape');
});
it('returns null otherwise', () => {
  expect(matchShortcut(k('b', false))).toBeNull();
  expect(matchShortcut(k('0'))).toBeNull();
  expect(matchShortcut(k('x'))).toBeNull();
});
