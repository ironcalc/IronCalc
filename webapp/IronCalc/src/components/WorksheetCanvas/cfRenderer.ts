import type { CfDataBar, CfIcon, IconSetType } from "@ironcalc/wasm";

// Width (in pixels) reserved on the left of a cell for an icon-set indicator.
export const ICON_AREA_WIDTH = 20;

// ---------------------------------------------------------------------------
// Border drawing helpers (extracted from WorksheetCanvas to keep it smaller)
// ---------------------------------------------------------------------------

function drawBorderLine(
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

export function renderDataBar(
  context: CanvasRenderingContext2D,
  x: number,
  y: number,
  width: number,
  height: number,
  dataBar: CfDataBar,
): void {
  const barWidth = Math.max(0, Math.round(dataBar.value * width));
  const vertPad = Math.max(2, Math.round(height * 0.15));
  context.save();
  context.fillStyle = dataBar.color;
  context.fillRect(x, y + vertPad, barWidth, height - 2 * vertPad);
  context.restore();
}

// ---------------------------------------------------------------------------
// Icon-set renderer
// ---------------------------------------------------------------------------

interface IconSpec {
  char: string;
  color: string;
}
// ←	↑	→	↓	↔	↕	↖	↗	↘	↙
// idx=0 = worst (lowest value), idx=N-1 = best (highest value)
const ARROW_CHARS = ["↓", "↘", "→", "↗", "↑"];
const ARROW_COLORS = ["#e43400", "#ffeb84", "#ffeb84", "#ffeb84","#84cb1f"];

function getIconSpec(set: IconSetType, index: number): IconSpec {
  switch (set) {
    case "Arrows3":
      return {
        char: ARROW_CHARS[[0, 2, 4][index]],
        color: ARROW_COLORS[[0, 2, 4][index]],
      };
    case "ArrowsGray3":
      return { char: ARROW_CHARS[[0, 2, 4][index]], color: "#808080" };
    case "Arrows4":
      return {
        char: ARROW_CHARS[[0, 1, 3, 4][index]],
        color: ARROW_COLORS[[0, 1, 3, 4][index]],
      };
    case "ArrowsGray4":
      return { char: ARROW_CHARS[[0, 1, 3, 4][index]], color: "#808080" };
    case "Arrows5":
      return { char: ARROW_CHARS[index], color: ARROW_COLORS[index] };
    case "ArrowsGray5":
      return { char: ARROW_CHARS[index], color: "#808080" };
    case "Triangles3":
      return {
        char: ["▼", "▬", "▲"][index],
        color: ["#f8696b", "#ffeb84", "#63be7b"][index],
      };
    case "TrafficLights3":
    case "TrafficLights3Rimmed":
      return {
        char: "●",
        color: ["#f8696b", "#ffeb84", "#63be7b"][index],
      };
    case "TrafficLights4":
      return {
        char: "●",
        color: ["#000000", "#f8696b", "#ffeb84", "#63be7b"][index],
      };
    case "Signs3":
      return {
        char: ["✖", "▲", "●"][index],
        color: ["#f8696b", "#ffeb84", "#63be7b"][index],
      };
    case "RedToBlack4":
      return {
        char: "●",
        color: ["#000000", "#808080", "#f66f00", "#e43400"][index],
      };
    case "Symbols3Circled":
    case "Symbols3Uncircled":
      return {
        char: ["✘", "!", "✔"][index],
        color: ["#f8696b", "#ffeb84", "#63be7b"][index],
      };
    case "Flags3":
      return {
        char: "⚑",
        color: ["#f8696b", "#ffeb84", "#63be7b"][index],
      };
    case "Stars3":
      return {
        char: index === 0 ? "☆" : "★",
        color: index === 2 ? "#ffd700" : "#c0c0c0",
      };
    case "Quarters5":
      return {
        char: ["○", "◔", "◑", "◕", "●"][index],
        color: "#808080",
      };
    case "Boxes5": {
      const colors5 = ["#f8696b", "#fc9f6e", "#ffeb84", "#b4d67e", "#63be7b"];
      return {
        char: "■",
        color: colors5[index],
      };
    }
    case "Ratings4": {
      const colors4 = ["#f8696b", "#ffeb84", "#b4d67e", "#63be7b"];
      return {
        char: "■",
        color: colors4[index],
      };
    }
    case "Ratings5": {
      const colors5 = ["#f8696b", "#fc9f6e", "#ffeb84", "#b4d67e", "#63be7b"];
      return {
        char: "■",
        color: colors5[index],
      };
    }
    default:
      return { char: "●", color: "#808080" };
  }
}

export function renderIcon(
  context: CanvasRenderingContext2D,
  x: number,
  y: number,
  height: number,
  icon: CfIcon,
): void {
  const { char, color } = getIconSpec(icon.set, icon.index);
  const fontSize = Math.min(Math.round(height * 0.55), 13);
  context.save();
  context.font = `${fontSize}px sans-serif`;
  context.fillStyle = color;
  context.textAlign = "center";
  context.textBaseline = "middle";
  context.fillText(char, x + ICON_AREA_WIDTH / 2, y + height / 2);
  context.restore();
}
