import { expect, it } from 'vitest';
import { fetchIntervalMs, shouldFetch } from './backgroundFetch';

it('converts minutes to ms, 0 or invalid = off', () => {
  expect(fetchIntervalMs(5)).toBe(300_000);
  expect(fetchIntervalMs(0)).toBeNull();
  expect(fetchIntervalMs(-1)).toBeNull();
  expect(fetchIntervalMs(Number.NaN)).toBeNull();
});

it('skips a tick while any op is busy or no repo/interval', () => {
  expect(shouldFetch(false, 5)).toBe(true);
  expect(shouldFetch(true, 5)).toBe(false);
  expect(shouldFetch(false, 0)).toBe(false);
});
