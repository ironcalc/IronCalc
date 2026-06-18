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
  context.save();
  context.strokeStyle = color;

  switch (style) {
    case "thin":
      context.lineWidth = 1;
      drawBorderLine(context, x1, y1, x2, y2);
      break;
    case "medium":
      context.lineWidth = 2;
      drawBorderLine(context, x1, y1, x2, y2);
      break;
    case "thick":
      context.lineWidth = 3;
      drawBorderLine(context, x1, y1, x2, y2);
      break;
    case "double":
      context.lineWidth = 1;
      if (isVertical) {
        drawBorderLine(context, x1 - 1, y1, x1 - 1, y2);
        drawBorderLine(context, x1 + 1, y1, x1 + 1, y2);
      } else {
        drawBorderLine(context, x1, y1 - 1, x2, y1 - 1);
        drawBorderLine(context, x1, y1 + 1, x2, y1 + 1);
      }
      break;
    case "dotted":
      context.lineWidth = 1;
      context.setLineDash([1, 2]);
      drawBorderLine(context, x1, y1, x2, y2);
      context.setLineDash([]);
      break;
    case "mediumdashed":
      context.lineWidth = 2;
      context.setLineDash([4, 2]);
      drawBorderLine(context, x1, y1, x2, y2);
      context.setLineDash([]);
      break;
    case "slantdashdot":
      context.lineWidth = 1;
      context.setLineDash([4, 2, 1, 2]);
      drawBorderLine(context, x1, y1, x2, y2);
      context.setLineDash([]);
      break;
    case "mediumdashdot":
      context.lineWidth = 2;
      context.setLineDash([4, 2, 1, 2]);
      drawBorderLine(context, x1, y1, x2, y2);
      context.setLineDash([]);
      break;
    case "mediumdashdotdot":
      context.lineWidth = 2;
      context.setLineDash([4, 2, 1, 2, 1, 2]);
      drawBorderLine(context, x1, y1, x2, y2);
      context.setLineDash([]);
      break;
    case "dashed":
      context.lineWidth = 1;
      context.setLineDash([4, 2]);
      drawBorderLine(context, x1, y1, x2, y2);
      context.setLineDash([]);
      break;
    default:
      context.lineWidth = 1;
      drawBorderLine(context, x1, y1, x2, y2);
  }
  context.restore();
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
