/** Interval in ms for a minutes setting; null when background fetch is off (0, negative or not a number). */
export function fetchIntervalMs(min: number): number | null {
  return Number.isFinite(min) && min > 0 ? min * 60_000 : null;
}

/**
 * A background fetch tick runs only when fetching is on, no foreground operation is
 * in flight and the previous background fetch has finished (ticks never overlap).
 */
export const shouldFetch = (busy: boolean, intervalMin: number, inFlight: boolean) =>
  !busy && !inFlight && fetchIntervalMs(intervalMin) !== null;
