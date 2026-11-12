import { expect, it } from 'vitest';
import { repoInitials, repoName } from './repoInitials';

it('uses first letters of the first two separated words', () => {
  expect(repoInitials('gk-platform')).toBe('GP');
  expect(repoInitials('my_cool.repo')).toBe('MC');
  expect(repoInitials('a-b-c')).toBe('AB');
});
it('falls back to the first two letters', () => {
  expect(repoInitials('gitsaurus')).toBe('GI');
  expect(repoInitials('x')).toBe('X');
  expect(repoInitials('-lead')).toBe('LE');
  expect(repoInitials('')).toBe('?');
});
it('repoName is the last path segment', () => {
  expect(repoName('/a/b/gk-platform')).toBe('gk-platform');
  expect(repoName('/a/b/gk-platform/')).toBe('gk-platform');
});
