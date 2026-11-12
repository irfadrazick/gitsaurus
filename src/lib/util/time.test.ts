import { describe, expect, it } from 'vitest';
import { showMarker, timeBucket } from './time';

const now = 1_800_000_000;
describe('timeBucket', () => {
  it('buckets', () => {
    expect(timeBucket(now - 30, now)).toBe('just now');
    expect(timeBucket(now - 3600, now)).toBe('1 hour ago');
    expect(timeBucket(now - 3 * 3600, now)).toBe('3 hours ago');
    expect(timeBucket(now - 8 * 86400, now)).toBe('a week ago');
    expect(timeBucket(now - 15 * 86400, now)).toBe('2 weeks ago');
    expect(timeBucket(now - 40 * 86400, now)).toBe('a month ago');
  });
  it('showMarker is true when the bucket changes', () => {
    expect(showMarker(undefined, now - 30, now)).toBe(true);
    expect(showMarker(now - 30, now - 40, now)).toBe(false);
    expect(showMarker(now - 30, now - 3600, now)).toBe(true);
  });
});
