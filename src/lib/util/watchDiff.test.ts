import { expect, it } from 'vitest';
import { defaultRemote, watchDiff } from './watchDiff';

it('watchDiff reports added and removed repos', () => {
  expect(watchDiff([], ['a', 'b'])).toEqual({ add: ['a', 'b'], remove: [] });
  expect(watchDiff(['a', 'b'], ['b', 'c'])).toEqual({ add: ['c'], remove: ['a'] });
  expect(watchDiff(['a'], ['a'])).toEqual({ add: [], remove: [] });
});
it('defaultRemote prefers origin', () => {
  expect(defaultRemote(['up', 'origin'])).toBe('origin');
  expect(defaultRemote(['up'])).toBe('up');
  expect(defaultRemote([])).toBeNull();
});
