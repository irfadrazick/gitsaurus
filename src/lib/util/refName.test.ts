import { expect, it } from 'vitest';
import { explainRefName } from './refName';

it('accepts ordinary names', () => {
  for (const n of ['main', 'feature/offline-maps', 'fix_1.2', 'v0.2.0', 'a/b/c']) expect(explainRefName(n)).toBeNull();
});

it('names the rule a bad name breaks', () => {
  expect(explainRefName('')).toBe('Enter a name.');
  expect(explainRefName('my branch')).toBe("Names can't contain spaces.");
  expect(explainRefName('-x')).toBe('Names can\'t start with "-".');
  expect(explainRefName('bad..name')).toBe('Names can\'t contain "..".');
  expect(explainRefName('a~b')).toBe("Names can't contain ~ ^ : ? * [ or \\.");
  expect(explainRefName('a@{1}')).toBe('Names can\'t contain "@{".');
  expect(explainRefName('feat.')).toBe('Names can\'t end with ".", "/" or ".lock".');
  expect(explainRefName('feat.lock')).toBe('Names can\'t end with ".", "/" or ".lock".');
  expect(explainRefName('a//b')).toBe('Each part between slashes must be non-empty and not start with ".".');
  expect(explainRefName('a/.hidden')).toBe('Each part between slashes must be non-empty and not start with ".".');
  expect(explainRefName('@')).toBe('"@" on its own isn\'t allowed.');
});
