import type { CfDataBar, CfIcon, CfRating, Color } from "@ironcalc/wasm";
import { ICON_PATH_SPECS } from "./lucideIconPaths";

// Width (in pixels) reserved on the left of a cell for an icon-set indicator.
export const ICON_AREA_WIDTH = 20;
// Left margin so icons don't sit too close to the cell border.
const ICON_LEFT_MARGIN = 3;

// ---------------------------------------------------------------------------
// Border drawing helpers (extracted from WorksheetCanvas to keep it smaller)
// ---------------------------------------------------------------------------

export function drawBorderLine(
  context: CanvasRenderingContext2D,
  x1: number,
  y1: number,
  x2: number,
  y2: number,
): void {
  context.beginPath();
  context.moveTo(x1, y1);
  context.lineTo(x2, y2);
  context.stroke();
}

// A border segment from (x1, y1) to (x2, y2). Vertical segments are told
// apart for the extension of wide lines at their ends, see `drawBorders`.
export type BorderSegment = [
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  isVertical: boolean,
];

// Whether a border of `style` leaves gaps: the band between the two lines
// of a "double" border, or the gaps of a dash pattern. Such a border is
// stroked over an underlay (see `underlayStyle`) in the colour the grid
// line under the border would be covered with, so the gaps show the cell's
// fill or the background rather than the grid lines running into the
// border, or nothing at all (the page behind the canvas).
export function hasGaps(style: string): boolean {
  return style === "double" || borderLine(style).dash.length > 0;
}

// The pseudo style of the underlay of a border of `style`: a solid line of
// the same width (one pixel for "double": the band between its two lines,
// which come as "double" segments already offset to their side of the edge
// and cut to length, see WorksheetCanvas.queueDoubleBorder).
export function underlayStyle(style: string): string {
  return `${UNDERLAY_PREFIX}${style}`;
}

const UNDERLAY_PREFIX = "underlay:";

// The line width and dash pattern of a border style
function borderLine(style: string): { lineWidth: number; dash: number[] } {
  if (style.startsWith(UNDERLAY_PREFIX)) {
    const base = style.slice(UNDERLAY_PREFIX.length);
    return {
      lineWidth: base === "double" ? 1 : borderLine(base).lineWidth,
      dash: [],
    };
  }
  switch (style) {
    case "medium":
      return { lineWidth: 2, dash: [] };
    case "thick":
      return { lineWidth: 3, dash: [] };
    case "dotted":
      return { lineWidth: 1, dash: [1, 2] };
    case "mediumdashed":
      return { lineWidth: 2, dash: [4, 2] };
    case "slantdashdot":
      return { lineWidth: 1, dash: [4, 2, 1, 2] };
    case "mediumdashdot":
      return { lineWidth: 2, dash: [4, 2, 1, 2] };
    case "mediumdashdotdot":
      return { lineWidth: 2, dash: [4, 2, 1, 2, 1, 2] };
    case "dashed":
      return { lineWidth: 1, dash: [4, 2] };
    default:
      // thin, double and anything unknown
      return { lineWidth: 1, dash: [] };
  }
}

function lineExtension(style: string, lineWidth: number): number {
  if (style === underlayStyle("double")) {
    // Cut to length with the lines, see WorksheetCanvas.queueDoubleBorder
    return 0;
  }
  if (lineWidth > 1) {
    return lineWidth / 2;
  }
  return style.startsWith(UNDERLAY_PREFIX) ? -0.5 : 0;
}

// Strokes `segments` with one border style and colour as a single path:
// the context is set up once and the segments share one stroke call, so a
// frame's thousands of grid lines cost a handful of strokes. (A dash pattern
// restarts at each segment, as it does for separately stroked lines.)
export function drawBorders(
  context: CanvasRenderingContext2D,
  style: string,
  color: string,
  segments: BorderSegment[],
): void {
  if (segments.length === 0) {
    return;
  }
  const { lineWidth, dash } = borderLine(style);
  // A segment runs between two grid line positions. A line wider than the
  // grid line overhangs it on both sides, so two such lines meeting at a
  // corner would leave the corner square outside both unpainted: each
  // segment is extended by half the line's full width at both ends, which
  // fills the corner, and squares off a free end over the grid line it ends
  // on, as Excel does. Thin lines are left alone: the crossing grid line
  // already covers their end pixel, and a double border arrives cut to
  // length. The underlay of a thin dashed line stops short of that pixel,
  // which it would otherwise half cover.
  const ext = lineExtension(style, lineWidth);
  context.save();
  context.strokeStyle = color;
  context.lineWidth = lineWidth;
  if (dash.length) {
    context.setLineDash(dash);
  }
  context.beginPath();
  for (const [x1, y1, x2, y2, isVertical] of segments) {
    if (isVertical) {
      context.moveTo(x1, y1 - ext);
      context.lineTo(x2, y2 + ext);
    } else {
      context.moveTo(x1 - ext, y1);
      context.lineTo(x2 + ext, y2);
    }
  }
  context.stroke();
  if (dash.length) {
    context.setLineDash([]);
  }
  context.restore();
}

// Strokes one border segment, see `drawBorders`
export function drawBorder(
  context: CanvasRenderingContext2D,
  style: string,
  color: string,
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  isVertical: boolean,
): void {
  drawBorders(context, style, color, [[x1, y1, x2, y2, isVertical]]);
}

// ---------------------------------------------------------------------------
// Data-bar renderer
// ---------------------------------------------------------------------------

function lightenColor(hex: string, amount: number): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  const lr = Math.round(r + (255 - r) * amount);
  const lg = Math.round(g + (255 - g) * amount);
  const lb = Math.round(b + (255 - b) * amount);
  return `#${lr.toString(16).padStart(2, "0")}${lg.toString(16).padStart(2, "0")}${lb.toString(16).padStart(2, "0")}`;
}

export function renderDataBar(
  context: CanvasRenderingContext2D,
  x: number,
  y: number,
  width: number,
  height: number,
  dataBar: CfDataBar,
  resolveColor: (color: Color) => string,
): void {
  const { value, axis_position, is_gradient } = dataBar;
  const positive_color = resolveColor(dataBar.positive_color);
  const negative_color = resolveColor(dataBar.negative_color);
  const vertPad = Math.max(2, Math.round(height * 0.15));
  const barY = y + vertPad;
  const barHeight = height - 2 * vertPad;
  const axisX = x + Math.round(axis_position * width);

  context.save();

  if (value >= axis_position) {
    // Positive bar: from axis to value
    const barLeft = axisX;
    const barRight = x + Math.round(value * width);
    const barWidth = Math.max(0, barRight - barLeft);
    if (barWidth > 0) {
      if (is_gradient && barWidth > 1) {
        const grad = context.createLinearGradient(barLeft, 0, barRight, 0);
        grad.addColorStop(0, lightenColor(positive_color, 0.6));
        grad.addColorStop(1, positive_color);
        context.fillStyle = grad;
      } else {
        context.fillStyle = positive_color;
      }
      context.fillRect(barLeft, barY, barWidth, barHeight);
    }
  } else {
    // Negative bar: from value to axis
    const barLeft = x + Math.round(value * width);
    const barRight = axisX;
    const barWidth = Math.max(0, barRight - barLeft);
    if (barWidth > 0) {
      if (is_gradient && barWidth > 1) {
        const grad = context.createLinearGradient(barLeft, 0, barRight, 0);
        grad.addColorStop(0, negative_color);
        grad.addColorStop(1, lightenColor(negative_color, 0.6));
        context.fillStyle = grad;
      } else {
        context.fillStyle = negative_color;
      }
      context.fillRect(barLeft, barY, barWidth, barHeight);
    }
  }

  context.restore();
}

// ---------------------------------------------------------------------------
// Icon-set renderer (Lucide SVG paths drawn via Path2D)
// ---------------------------------------------------------------------------

function drawLucideIcon(
  context: CanvasRenderingContext2D,
  iconName: string,
  color: string,
  cx: number,
  cy: number,
  size: number,
): void {
  const spec = ICON_PATH_SPECS[iconName];
  if (!spec) {
    return;
  }

  const scale = size / 24;
  context.save();
  context.translate(cx - size / 2, cy - size / 2);
  context.scale(scale, scale);

  if (spec.flipY) {
    context.translate(0, 24);
    context.scale(1, -1);
  }

  context.strokeStyle = color;
  context.fillStyle = color;
  context.lineWidth = (spec.strokeWidth ?? 1.5) / scale;
  context.lineCap = "round";
  context.lineJoin = "round";

  for (const d of spec.paths) {
    const path = new Path2D(d);
    if (spec.fill ?? !spec.stroke) {
      context.fill(path);
    }
    if (spec.stroke) {
      context.stroke(path);
    }
  }

  context.restore();
}

export function renderIcon(
  context: CanvasRenderingContext2D,
  x: number,
  y: number,
  height: number,
  icon: CfIcon,
  size: number,
  resolveColor: (color: Color) => string,
): void {
  context.save();
  drawLucideIcon(
    context,
    icon.icon,
    resolveColor(icon.color),
    x + ICON_LEFT_MARGIN + size / 2,
    y + height / 2,
    size,
  );
  context.restore();
}

export function renderRating(
  context: CanvasRenderingContext2D,
  x: number,
  y: number,
  height: number,
  rating: CfRating,
  size: number,
  resolveColor: (color: Color) => string,
): void {
  context.save();
  const color = resolveColor(rating.color);
  for (let i = 0; i < rating.count; i++) {
    drawLucideIcon(
      context,
      rating.icon,
      color,
      x + ICON_LEFT_MARGIN + size / 2 + i * size,
      y + height / 2,
      size,
    );
  }
  context.restore();
}
