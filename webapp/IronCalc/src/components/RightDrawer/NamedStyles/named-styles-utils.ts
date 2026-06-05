import type { CellStyle, Color, Model } from "@ironcalc/wasm";
import type { CSSProperties } from "react";

const BORDER_WIDTH: Record<string, string> = {
  thin: "1px",
  medium: "2px",
  thick: "3px",
  double: "3px",
  dotted: "1px",
  slantdashdot: "1px",
  mediumdashed: "2px",
  mediumdashdotdot: "2px",
  mediumdashdot: "2px",
};

const BORDER_CSS_STYLE: Record<string, string> = {
  thin: "solid",
  medium: "solid",
  thick: "solid",
  double: "double",
  dotted: "dotted",
  slantdashdot: "dashed",
  mediumdashed: "dashed",
  mediumdashdotdot: "dashed",
  mediumdashdot: "dashed",
};

interface BorderItem {
  style: string;
  color?: Color;
}

function getBorderValue(
  model: Model,
  item: BorderItem | undefined,
): string | undefined {
  if (!item?.style) {
    return undefined;
  }
  const width = BORDER_WIDTH[item.style] ?? "1px";
  const cssStyle = BORDER_CSS_STYLE[item.style] ?? "solid";
  const color = model.resolveColor(item.color) || "currentColor";
  return `${width} ${cssStyle} ${color}`;
}

export function getTileStyle(model: Model, style: CellStyle): CSSProperties {
  const decorations: string[] = [];
  if (style.font.u) {
    decorations.push("underline");
  }
  if (style.font.strike) {
    decorations.push("line-through");
  }
  return {
    backgroundColor: model.resolveColor(style.fill.color) || undefined,
    color: model.resolveColor(style.font.color) || undefined,
    fontWeight: style.font.b ? "bold" : undefined,
    fontStyle: style.font.i ? "italic" : undefined,
    textDecoration: decorations.length > 0 ? decorations.join(" ") : undefined,
    borderTop: getBorderValue(model, style.border?.top),
    borderRight: getBorderValue(model, style.border?.right),
    borderBottom: getBorderValue(model, style.border?.bottom),
    borderLeft: getBorderValue(model, style.border?.left),
  };
}
