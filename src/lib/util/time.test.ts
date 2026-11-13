import { expect, it } from 'vitest';
import { shortAge } from './time';

const now = 1_800_000_000;
it('shortAge', () => {
  expect(shortAge(now - 30, now)).toBe('now');
  expect(shortAge(now + 30, now)).toBe('now'); // clock skew
  expect(shortAge(now - 5 * 60, now)).toBe('5m');
  expect(shortAge(now - 3600, now)).toBe('1h');
  expect(shortAge(now - 10 * 3600, now)).toBe('10h');
  expect(shortAge(now - 86400, now)).toBe('1d');
  expect(shortAge(now - 8 * 86400, now)).toBe('1w');
  expect(shortAge(now - 40 * 86400, now)).toBe('1mo');
  expect(shortAge(now - 400 * 86400, now)).toBe('1y');
});
