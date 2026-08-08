import type {
  CellStyle,
  Color,
  FmtSettings,
  HorizontalAlignment,
  Model,
  VerticalAlignment,
} from "@ironcalc/wasm";
import type { CSSProperties } from "react";
import { NumberFormats } from "../../FormatMenu/formatUtil";

export const HORIZONTAL_JUSTIFY: Partial<Record<HorizontalAlignment, string>> =
  {
    left: "flex-start",
    center: "center",
    right: "flex-end",
  };

export const VERTICAL_ALIGN_ITEMS: Partial<Record<VerticalAlignment, string>> =
  {
    top: "flex-start",
    center: "center",
    bottom: "flex-end",
  };

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

// Faint outline for preview sides without a real border
export const FAINT_PREVIEW_BORDER =
  "1px solid color-mix(in srgb, var(--palette-common-black) 12%, transparent)";

export function borderStyleToCss(style: string, color: string): string {
  const width = BORDER_WIDTH[style] ?? "1px";
  const cssStyle = BORDER_CSS_STYLE[style] ?? "solid";
  return `${width} ${cssStyle} ${color}`;
}

function getBorderValue(
  model: Model,
  item: BorderItem | undefined,
): string | undefined {
  if (!item?.style) {
    return undefined;
  }
  const color = model.resolveColor(item.color) || "currentColor";
  return borderStyleToCss(item.style, color);
}

export function getPreviewText(
  numFmt: string,
  formatOptions: FmtSettings,
  t: (key: string) => string,
): string {
  if (numFmt === NumberFormats.AUTO) {
    return "Aa";
  }
  if (numFmt === formatOptions.number_fmt) {
    return "#";
  }
  if (numFmt === NumberFormats.PERCENTAGE) {
    return "%";
  }
  if (numFmt === NumberFormats.CURRENCY_EUR) {
    return t("toolbar.format_menu.currency_eur_example");
  }
  if (numFmt === NumberFormats.CURRENCY_USD) {
    return t("toolbar.format_menu.currency_usd_example");
  }
  if (numFmt === NumberFormats.CURRENCY_GBP) {
    return t("toolbar.format_menu.currency_gbp_example");
  }
  if (
    numFmt === formatOptions.short_date ||
    numFmt === formatOptions.long_date
  ) {
    return "31/12";
  }
  return "C";
}

export function getTileStyle(model: Model, style: CellStyle): CSSProperties {
  const decorations: string[] = [];
  if (style.font.u) {
    decorations.push("underline");
  }
  if (style.font.strike) {
    decorations.push("line-through");
  }
  const tileStyle: CSSProperties = {
    backgroundColor: model.resolveColor(style.fill.color) || undefined,
    color: model.resolveColor(style.font.color) || undefined,
    fontWeight: style.font.b ? "bold" : undefined,
    fontStyle: style.font.i ? "italic" : undefined,
    textDecoration: decorations.length > 0 ? decorations.join(" ") : undefined,
    justifyContent: style.alignment?.horizontal
      ? HORIZONTAL_JUSTIFY[style.alignment.horizontal]
      : undefined,
    alignItems: style.alignment?.vertical
      ? VERTICAL_ALIGN_ITEMS[style.alignment.vertical]
      : undefined,
  };

  const top = getBorderValue(model, style.border?.top);
  const right = getBorderValue(model, style.border?.right);
  const bottom = getBorderValue(model, style.border?.bottom);
  const left = getBorderValue(model, style.border?.left);
  // Draw every side explicitly (faint fallback where unset) and drop the
  // box-shadow so it doesn't double up with real borders.
  if (top || right || bottom || left) {
    tileStyle.boxShadow = "none";
    tileStyle.borderTop = top ?? FAINT_PREVIEW_BORDER;
    tileStyle.borderRight = right ?? FAINT_PREVIEW_BORDER;
    tileStyle.borderBottom = bottom ?? FAINT_PREVIEW_BORDER;
    tileStyle.borderLeft = left ?? FAINT_PREVIEW_BORDER;
  }

  return tileStyle;
}
