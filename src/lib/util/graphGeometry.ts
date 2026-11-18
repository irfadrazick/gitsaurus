// Pure geometry for the commit graph (spec §6.2). Components stay thin and draw what these return.

export const ROW_H = 32;
export const LANE_W = 22;
export const LANE_X0 = 16;
export const NODE_R = 11;
/** Selection ring around a commit node; LANE_X0 and cellWidth leave room for it (plus half its stroke). */
export const SEL_RING_R = NODE_R + 3.5;
export const MERGE_R = 6;
export const ELBOW_R = 8;
export const LINE_W = 2;
export const OVERSCAN = 10;

/** Lane palette; mirrors --lane-0..9 in app.css (hex so it can carry an alpha byte). */
export const palette = [
  '#3fb9a6', '#4f8ff7', '#a371f7', '#e05cc4', '#f0883e',
  '#e3b341', '#f85149', '#56d364', '#79c0ff', '#d2a8ff',
] as const;

/** Colour for a backend colour id (ids are unbounded; the palette wraps). */
export const laneColor = (color: number): string => palette[((color % palette.length) + palette.length) % palette.length];

/** Centre x of a lane inside a graph cell. */
export const laneX = (lane: number): number => LANE_X0 + lane * LANE_W;

/** Width of a row's graph SVG for a row `width` (lanes in use). */
export const cellWidth = (width: number): number => width * LANE_W + 10;

/** Narrowest graph column: one lane plus room for the GRAPH header label. */
export const GRAPH_COL_MIN = 64;

/** Graph column width: fits `maxLanes`, capped by the user-resizable `cap`, never below GRAPH_COL_MIN. */
export const graphColumnWidth = (maxLanes: number, cap: number): number =>
  Math.max(GRAPH_COL_MIN, Math.min(cellWidth(maxLanes), cap));

export type SegmentKind = 'into' | 'out' | 'pass';

/**
 * SVG path for one line of a row whose node sits in `nodeLane`.
 * - pass: straight down through the row in `lane`
 * - into: from the top of `lane` to the node; an elbow (vertical, quarter circle r=8, horizontal) when lane != nodeLane
 * - out: from the node to the bottom of `lane`, with the mirrored elbow
 */
export function segmentPath(kind: SegmentKind, lane: number, nodeLane: number, rowH = ROW_H): string {
  const x = laneX(lane);
  const nx = laneX(nodeLane);
  const cy = rowH / 2;
  const r = ELBOW_R;
  if (kind === 'pass') return `M ${x} 0 L ${x} ${rowH}`;
  if (kind === 'into') {
    if (lane === nodeLane) return `M ${x} 0 L ${x} ${cy}`;
    const dir = Math.sign(nx - x);
    // Heading down, then turning toward the node: a left turn (sweep 0) when the node is to the right.
    const sweep = dir > 0 ? 0 : 1;
    return `M ${x} 0 L ${x} ${cy - r} A ${r} ${r} 0 0 ${sweep} ${x + dir * r} ${cy} L ${nx} ${cy}`;
  }
  if (lane === nodeLane) return `M ${x} ${cy} L ${x} ${rowH}`;
  const dir = Math.sign(x - nx);
  // Heading away from the node, then turning down: a right turn (sweep 1) when heading right.
  const sweep = dir > 0 ? 1 : 0;
  return `M ${nx} ${cy} L ${x - dir * r} ${cy} A ${r} ${r} 0 0 ${sweep} ${x} ${cy + r} L ${x} ${rowH}`;
}

/** Rows [start, end) to render for a scroll position, including `overscan` rows on each side. */
export function visibleRange(
  scrollTop: number,
  viewportH: number,
  rowH: number,
  total: number,
  overscan: number,
): { start: number; end: number } {
  if (total <= 0) return { start: 0, end: 0 };
  const top = Math.max(0, scrollTop);
  const first = Math.floor(top / rowH);
  const last = Math.ceil((top + Math.max(0, viewportH)) / rowH);
  const start = Math.min(total, Math.max(0, first - overscan));
  const end = Math.min(total, last + overscan);
  return { start, end: Math.max(start, end) };
}

/** Page numbers covering rows [start, end). */
export function pagesFor(start: number, end: number, pageSize: number): number[] {
  if (end <= start) return [];
  const pages: number[] = [];
  for (let p = Math.floor(start / pageSize); p <= Math.floor((end - 1) / pageSize); p++) pages.push(p);
  return pages;
}

/** The pages of `pages` that still hold rows in a graph of `total` rows. */
export function pagesWithin(pages: number[], total: number, pageSize: number): number[] {
  return pages.filter((p) => p * pageSize < total);
}

const fullyVisible = (scrollTop: number, viewportH: number, rowTop: number, rowH: number) =>
  rowTop >= scrollTop && rowTop + rowH <= scrollTop + viewportH;

/** Scroll position that reveals a row: unchanged when it is fully visible, else the row is centred. */
export function revealScrollTop(scrollTop: number, viewportH: number, rowTop: number, rowH: number): number {
  if (fullyVisible(scrollTop, viewportH, rowTop, rowH)) return scrollTop;
  return Math.max(0, rowTop - viewportH / 2 + rowH / 2);
}

/** Smallest scroll change that brings a row fully into view (keyboard selection). */
export function keepVisibleScrollTop(scrollTop: number, viewportH: number, rowTop: number, rowH: number): number {
  if (rowTop < scrollTop) return Math.max(0, rowTop);
  if (rowTop + rowH > scrollTop + viewportH) return Math.max(0, rowTop + rowH - viewportH);
  return scrollTop;
}

/** Next (dir 1) or previous (dir -1) match position, wrapping; -1 when there are no matches. */
export function stepMatch(count: number, current: number, dir: 1 | -1): number {
  if (count <= 0) return -1;
  if (current < 0) return dir > 0 ? 0 : count - 1;
  return (current + dir + count) % count;
}

/**
 * Ruling R8: the backend seeds lane 0 with HEAD before row 0, so every row above `headIndex` carries
 * lane 0 in `pass` and the HEAD row carries lane 0 in `into`. These "seed segments" are the WIP -> HEAD
 * connector. True when the given segment of row `index` is one of them.
 */
export function seedSegment(index: number, headIndex: number | null, kind: 'into' | 'pass', lane: number): boolean {
  if (headIndex === null || lane !== 0) return false;
  return kind === 'pass' ? index < headIndex : index === headIndex;
}

export type SegmentStyle = 'solid' | 'dashed' | 'hidden';

/** Seed segments are dashed under a WIP row and hidden without one (they would dangle from the list top). */
export const seedStyle = (isSeed: boolean, wip: boolean): SegmentStyle =>
  !isSeed ? 'solid' : wip ? 'dashed' : 'hidden';
