import { describe, expect, it } from 'vitest';
import {
  cellWidth,
  graphColumnWidth,
  keepVisibleScrollTop,
  laneColor,
  laneX,
  pagesFor,
  revealScrollTop,
  seedSegment,
  seedStyle,
  segmentPath,
  stepMatch,
  tint,
  visibleRange,
} from './graphGeometry';

describe('lane geometry', () => {
  it('laneX is 14 + lane * 22', () => {
    expect(laneX(0)).toBe(14);
    expect(laneX(1)).toBe(36);
    expect(laneX(4)).toBe(102);
  });
  it('cellWidth is width * 22 + 8', () => {
    expect(cellWidth(1)).toBe(30);
    expect(cellWidth(3)).toBe(74);
  });
  it('graph column fits max lanes up to the cap, never below 64 px (room for the header)', () => {
    expect(graphColumnWidth(3, 400)).toBe(74);
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
  it('tint appends an alpha byte for the given opacity', () => {
    expect(tint(0, 0.12)).toBe(`${laneColor(0)}1f`);
    expect(tint(0, 0.22)).toBe(`${laneColor(0)}38`);
    expect(tint(12, 1)).toBe(`${laneColor(2)}ff`);
  });
});

describe('segmentPath', () => {
  it('pass runs straight through the row', () => {
    expect(segmentPath('pass', 2, 0)).toBe('M 58 0 L 58 36');
  });
  it('into and out in the node lane are straight halves', () => {
    expect(segmentPath('into', 1, 1)).toBe('M 36 0 L 36 18');
    expect(segmentPath('out', 1, 1)).toBe('M 36 18 L 36 36');
  });
  it('into from a lane to the left: down, quarter turn (r 8), across to the node', () => {
    expect(segmentPath('into', 0, 1)).toBe('M 14 0 L 14 10 A 8 8 0 0 0 22 18 L 36 18');
  });
  it('into from a lane to the right turns the other way', () => {
    expect(segmentPath('into', 2, 0)).toBe('M 58 0 L 58 10 A 8 8 0 0 1 50 18 L 14 18');
  });
  it('out to a lane to the right: across, quarter turn down, to the bottom', () => {
    expect(segmentPath('out', 1, 0)).toBe('M 14 18 L 28 18 A 8 8 0 0 1 36 26 L 36 36');
  });
  it('out to a lane to the left', () => {
    expect(segmentPath('out', 0, 2)).toBe('M 58 18 L 22 18 A 8 8 0 0 0 14 26 L 14 36');
  });
  it('honours a custom row height', () => {
    expect(segmentPath('pass', 0, 0, 40)).toBe('M 14 0 L 14 40');
    expect(segmentPath('out', 1, 0, 40)).toBe('M 14 20 L 28 20 A 8 8 0 0 1 36 28 L 36 40');
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
