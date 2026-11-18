import { describe, expect, it } from 'vitest';
import {
  cellWidth,
  graphColumnWidth,
  keepVisibleScrollTop,
  LINE_W,
  ROW_H,
  SEL_RING_R,
  laneColor,
  laneX,
  pagesFor,
  pagesWithin,
  revealScrollTop,
  seedSegment,
  seedStyle,
  segmentPath,
  stepMatch,
  visibleRange,
} from './graphGeometry';

describe('lane geometry', () => {
  it('laneX is 16 + lane * 22', () => {
    expect(laneX(0)).toBe(16);
    expect(laneX(1)).toBe(38);
    expect(laneX(4)).toBe(104);
  });
  it('cellWidth is width * 22 + 10', () => {
    expect(cellWidth(1)).toBe(32);
    expect(cellWidth(3)).toBe(76);
  });
  it('the selection ring fits inside the cell in the first and last lanes', () => {
    const outer = SEL_RING_R + LINE_W / 2;
    for (const width of [1, 3, 8]) {
      expect(laneX(0) - outer).toBeGreaterThanOrEqual(0);
      expect(laneX(width - 1) + outer).toBeLessThanOrEqual(cellWidth(width));
    }
    expect(ROW_H / 2 - outer).toBeGreaterThanOrEqual(0);
  });
  it('graph column fits max lanes up to the cap, never below 64 px (room for the header)', () => {
    expect(graphColumnWidth(3, 400)).toBe(76);
    expect(graphColumnWidth(30, 200)).toBe(200);
    expect(graphColumnWidth(0, 200)).toBe(64);
    expect(graphColumnWidth(1, 200)).toBe(64);
    expect(graphColumnWidth(5, 10)).toBe(64);
  });
  it('palette wraps by color % 10', () => {
    expect(laneColor(0)).toBe(laneColor(10));
    expect(laneColor(13)).toBe(laneColor(3));
    expect(laneColor(1)).not.toBe(laneColor(2));
    expect(laneColor(0)).toMatch(/^#[0-9a-f]{6}$/);
  });
});

describe('segmentPath', () => {
  it('pass runs straight through the row', () => {
    expect(segmentPath('pass', 2, 0)).toBe('M 60 0 L 60 32');
  });
  it('into and out in the node lane are straight halves', () => {
    expect(segmentPath('into', 1, 1)).toBe('M 38 0 L 38 16');
    expect(segmentPath('out', 1, 1)).toBe('M 38 16 L 38 32');
  });
  it('into from a lane to the left: down, quarter turn (r 8), across to the node', () => {
    expect(segmentPath('into', 0, 1)).toBe('M 16 0 L 16 8 A 8 8 0 0 0 24 16 L 38 16');
  });
  it('into from a lane to the right turns the other way', () => {
    expect(segmentPath('into', 2, 0)).toBe('M 60 0 L 60 8 A 8 8 0 0 1 52 16 L 16 16');
  });
  it('out to a lane to the right: across, quarter turn down, to the bottom', () => {
    expect(segmentPath('out', 1, 0)).toBe('M 16 16 L 30 16 A 8 8 0 0 1 38 24 L 38 32');
  });
  it('out to a lane to the left', () => {
    expect(segmentPath('out', 0, 2)).toBe('M 60 16 L 24 16 A 8 8 0 0 0 16 24 L 16 32');
  });
  it('honours a custom row height', () => {
    expect(segmentPath('pass', 0, 0, 40)).toBe('M 16 0 L 16 40');
    expect(segmentPath('out', 1, 0, 40)).toBe('M 16 20 L 30 20 A 8 8 0 0 1 38 28 L 38 40');
  });
});

describe('visibleRange', () => {
  it('covers the viewport plus overscan, clamped to the list', () => {
    expect(visibleRange(0, 360, 36, 1000, 10)).toEqual({ start: 0, end: 20 });
    expect(visibleRange(3600, 360, 36, 1000, 10)).toEqual({ start: 90, end: 120 });
    expect(visibleRange(35990, 360, 36, 1000, 10)).toEqual({ start: 989, end: 1000 });
  });
  it('rounds partial rows outward', () => {
    expect(visibleRange(18, 36, 36, 1000, 0)).toEqual({ start: 0, end: 2 });
  });
  it('handles negative scroll (rubber band) and empty lists', () => {
    expect(visibleRange(-50, 360, 36, 1000, 10)).toEqual({ start: 0, end: 20 });
    expect(visibleRange(0, 360, 36, 0, 10)).toEqual({ start: 0, end: 0 });
  });
});

describe('pagesWithin', () => {
  it('keeps the pages that still hold rows after a reload', () => {
    expect(pagesWithin([0, 1], 450, 200)).toEqual([0, 1]);
    expect(pagesWithin([2, 3], 450, 200)).toEqual([2]);
    expect(pagesWithin([1], 200, 200)).toEqual([]);
    expect(pagesWithin([0], 0, 200)).toEqual([]);
  });
});

describe('pagesFor', () => {
  it('lists pages touched by [start, end)', () => {
    expect(pagesFor(0, 20, 200)).toEqual([0]);
    expect(pagesFor(190, 210, 200)).toEqual([0, 1]);
    expect(pagesFor(400, 400, 200)).toEqual([]);
  });
});

describe('scroll helpers', () => {
  it('revealScrollTop leaves a fully visible row alone, else centres it', () => {
    expect(revealScrollTop(0, 360, 72, 36)).toBe(0);
    expect(revealScrollTop(0, 360, 3600, 36)).toBe(3600 - 180 + 18);
    expect(revealScrollTop(1000, 360, 36, 36)).toBe(0);
  });
  it('keepVisibleScrollTop scrolls minimally', () => {
    expect(keepVisibleScrollTop(0, 360, 72, 36)).toBe(0);
    expect(keepVisibleScrollTop(0, 360, 360, 36)).toBe(36);
    expect(keepVisibleScrollTop(500, 360, 468, 36)).toBe(468);
  });
});

describe('stepMatch', () => {
  it('wraps forward and backward', () => {
    expect(stepMatch(3, 0, 1)).toBe(1);
    expect(stepMatch(3, 2, 1)).toBe(0);
    expect(stepMatch(3, 0, -1)).toBe(2);
    expect(stepMatch(3, -1, 1)).toBe(0);
    expect(stepMatch(0, -1, 1)).toBe(-1);
  });
});

describe('seed segments (R8: the HEAD lane seeded above row 0 is the WIP -> HEAD connector)', () => {
  it('head at 0: into lane 0 on row 0 is a seed', () => {
    expect(seedSegment(0, 0, 'into', 0)).toBe(true);
    expect(seedSegment(1, 0, 'into', 0)).toBe(false);
    expect(seedSegment(1, 0, 'pass', 0)).toBe(false);
  });
  it('head at 3: pass lane 0 above it and into lane 0 at it are seeds, nothing below', () => {
    for (const i of [0, 1, 2]) expect(seedSegment(i, 3, 'pass', 0)).toBe(true);
    expect(seedSegment(3, 3, 'into', 0)).toBe(true);
    expect(seedSegment(3, 3, 'pass', 0)).toBe(false);
    expect(seedSegment(4, 3, 'pass', 0)).toBe(false);
    expect(seedSegment(2, 3, 'into', 0)).toBe(false);
  });
  it('non-zero lanes are never seeds', () => {
    expect(seedSegment(0, 3, 'pass', 1)).toBe(false);
    expect(seedSegment(3, 3, 'into', 2)).toBe(false);
  });
  it('headIndex null: never', () => {
    expect(seedSegment(0, null, 'into', 0)).toBe(false);
    expect(seedSegment(0, null, 'pass', 0)).toBe(false);
  });
  it('seeds are dashed under a WIP row and hidden without one; other segments stay solid', () => {
    expect(seedStyle(true, true)).toBe('dashed');
    expect(seedStyle(true, false)).toBe('hidden');
    expect(seedStyle(false, true)).toBe('solid');
    expect(seedStyle(false, false)).toBe('solid');
  });
});
