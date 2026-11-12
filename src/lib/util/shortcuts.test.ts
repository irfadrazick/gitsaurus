import { expect, it } from 'vitest';
import { matchShortcut } from './shortcuts';

const k = (key: string, meta = true, shift = false) => ({ key, metaKey: meta, ctrlKey: false, altKey: false, shiftKey: shift });

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

it('maps the git shortcuts, with and without shift', () => {
  expect(matchShortcut(k('p'))).toBe('push');
  // With shift held, browsers report the key in upper case.
  expect(matchShortcut(k('P', true, true))).toBe('pull');
  expect(matchShortcut(k('F', true, true))).toBe('fetch');
  expect(matchShortcut(k('N', true, true))).toBe('newBranch');
  expect(matchShortcut(k('S', true, true))).toBe('stageAll');
  expect(matchShortcut(k('U', true, true))).toBe('unstageAll');
});

it('shift does not change the plain shortcuts into something else', () => {
  expect(matchShortcut(k('B', true, true))).toBeNull();
  expect(matchShortcut(k('f'))).toBe('search');
});
