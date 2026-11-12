import { expect, it } from 'vitest';
import { fetchIntervalMs, shouldFetch } from './backgroundFetch';

it('converts minutes to ms, 0 or invalid = off', () => {
  expect(fetchIntervalMs(5)).toBe(300_000);
  expect(fetchIntervalMs(0)).toBeNull();
  expect(fetchIntervalMs(-1)).toBeNull();
  expect(fetchIntervalMs(Number.NaN)).toBeNull();
});

it('skips a tick while any op is busy, a fetch is in flight or the interval is off', () => {
  expect(shouldFetch(false, 5, false)).toBe(true);
  expect(shouldFetch(true, 5, false)).toBe(false);
  expect(shouldFetch(false, 0, false)).toBe(false);
  // Ticks never overlap: the previous background fetch is still running.
  expect(shouldFetch(false, 5, true)).toBe(false);
});
