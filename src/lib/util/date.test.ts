import { describe, expect, it } from 'vitest';
import { formatDate } from './date';

const ts = (...a: [number, number, number, number, number]) => new Date(...a).getTime() / 1000;

describe('formatDate', () => {
  it("follows the given locale's date and time conventions", () => {
    expect(formatDate(ts(2024, 0, 5, 15, 7), 'en-US')).toMatch(/^Jan 5, 2024,? 3:07\sPM$/);
    expect(formatDate(ts(2024, 0, 5, 15, 7), 'en-GB')).toMatch(/^5 Jan 2024,? 15:07$/);
  });
  it('has no "@" separator', () => {
    expect(formatDate(ts(2024, 10, 12, 0, 5), 'en-US')).not.toContain('@');
  });
});
