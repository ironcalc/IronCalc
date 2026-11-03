import type { Anchor, AnchorBehavior } from "@ironcalc/wasm";

// English Metric Units per CSS pixel at 96 dpi
const EMU_PER_PIXEL = 9525;

/** A chart cannot be resized below this, in pixels. */
export const MIN_CHART_SIZE = 50;

/** The part of a row or column axis the chart geometry needs. */
export interface AxisLike {
  /** Offset of the leading edge of `index` (1-based) */
  start(index: number): number;
  /** The index whose span contains `offset` */
  at(offset: number): number;
}

export interface Axes {
  rows: AxisLike;
  columns: AxisLike;
}

/** A rectangle in sheet pixels: (0, 0) is the corner of cell A1. */
export interface SheetRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** The handles of the selection frame, by compass point. */
export type Handle = "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";

export const HANDLES: Handle[] = ["nw", "n", "ne", "e", "se", "s", "sw", "w"];

export const HANDLE_CURSORS: Record<Handle, string> = {
  n: "ns-resize",
  s: "ns-resize",
  e: "ew-resize",
  w: "ew-resize",
  ne: "nesw-resize",
  sw: "nesw-resize",
  nw: "nwse-resize",
  se: "nwse-resize",
};

/** Where an anchor's frame is, in sheet pixels. */
export function sheetRectFromAnchor(axes: Axes, anchor: Anchor): SheetRect {
  const x =
    axes.columns.start(anchor.from.column) +
    anchor.from.column_offset / EMU_PER_PIXEL;
  const y =
    axes.rows.start(anchor.from.row) + anchor.from.row_offset / EMU_PER_PIXEL;
  const right =
    axes.columns.start(anchor.to.column) +
    anchor.to.column_offset / EMU_PER_PIXEL;
  const bottom =
    axes.rows.start(anchor.to.row) + anchor.to.row_offset / EMU_PER_PIXEL;
  return {
    x,
    y,
    width: Math.max(0, right - x),
    height: Math.max(0, bottom - y),
  };
}

/** The anchor whose frame is `rect`: each corner as a cell and an offset into it. */
export function anchorFromSheetRect(
  axes: Axes,
  rect: SheetRect,
  behavior: AnchorBehavior = "MoveAndResize",
): Anchor {
  const point = (x: number, y: number) => {
    const column = axes.columns.at(x);
    const row = axes.rows.at(y);
    return {
      row,
      column,
      row_offset: Math.round(
        Math.max(0, y - axes.rows.start(row)) * EMU_PER_PIXEL,
      ),
      column_offset: Math.round(
        Math.max(0, x - axes.columns.start(column)) * EMU_PER_PIXEL,
      ),
    };
  };
  return {
    from: point(rect.x, rect.y),
    to: point(rect.x + rect.width, rect.y + rect.height),
    behavior,
  };
}

/** `start` moved by (dx, dy). */
export function movedRect(start: SheetRect, dx: number, dy: number): SheetRect {
  return { ...start, x: start.x + dx, y: start.y + dy };
}

/**
 * `start` resized by dragging `handle` by (dx, dy): the opposite edge stays
 * put and the size never drops below `MIN_CHART_SIZE`.
 */
export function resizedRect(
  start: SheetRect,
  handle: Handle,
  dx: number,
  dy: number,
): SheetRect {
  let { x, y, width, height } = start;
  if (handle.includes("e")) {
    width = Math.max(MIN_CHART_SIZE, width + dx);
  }
  if (handle.includes("s")) {
    height = Math.max(MIN_CHART_SIZE, height + dy);
  }
  if (handle.includes("w")) {
    const right = x + width;
    x = Math.min(x + dx, right - MIN_CHART_SIZE);
    width = right - x;
  }
  if (handle.includes("n")) {
    const bottom = y + height;
    y = Math.min(y + dy, bottom - MIN_CHART_SIZE);
    height = bottom - y;
  }
  return { x, y, width, height };
}
