import { describe, expect, it } from 'vitest';
import { formatDate } from './date';

const ts = (...a: [number, number, number, number, number]) => new Date(...a).getTime() / 1000;

describe('formatDate', () => {
  it('formats MM/DD/YYYY @ h:mm A in local time', () => {
    expect(formatDate(ts(2024, 0, 5, 15, 7))).toBe('01/05/2024 @ 3:07 PM');
  });
  it('uses 12 AM / 12 PM at midnight and noon', () => {
    expect(formatDate(ts(2024, 10, 12, 0, 5))).toBe('11/12/2024 @ 12:05 AM');
    expect(formatDate(ts(2024, 10, 12, 12, 0))).toBe('11/12/2024 @ 12:00 PM');
  });
  it('does not pad the hour', () => {
    expect(formatDate(ts(2024, 5, 30, 9, 45))).toBe('06/30/2024 @ 9:45 AM');
  });
});
