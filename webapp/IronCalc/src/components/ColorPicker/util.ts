import {
  type Color,
  hexWithTintToRgb,
  type IronCalcTheme,
} from "@ironcalc/wasm";

export function themeBaseColors(theme: IronCalcTheme): string[] {
  return [
    theme.lt1,
    theme.dk1,
    theme.lt2,
    theme.dk2,
    theme.accent1,
    theme.accent2,
    theme.accent3,
    theme.accent4,
    theme.accent5,
    theme.accent6,
  ];
}

// FIXME:
const TINT_PATTERNS = [
  [-0.05, -0.15, -0.25, -0.35, -0.5], // 0: lt1
  [0.5, 0.35, 0.25, 0.15, 0.05], // 1: dk1
  [-0.1, -0.25, -0.5, -0.75, -0.9], // 2: lt2
  [0.8, 0.6, 0.4, -0.25, -0.5], // 3: dk2
  [0.8, 0.6, 0.4, -0.25, -0.5], // 4–9: accents
  [0.8, 0.6, 0.4, -0.25, -0.5],
  [0.8, 0.6, 0.4, -0.25, -0.5],
  [0.8, 0.6, 0.4, -0.25, -0.5],
  [0.8, 0.6, 0.4, -0.25, -0.5],
  [0.8, 0.6, 0.4, -0.25, -0.5],
];

function themeToneValues(): [number, number][][] {
  return TINT_PATTERNS.map((tints, index) =>
    tints.map((tint) => [index, tint]),
  );
}

export function computeThemeGrid(
  theme: IronCalcTheme,
): { hex: string; color: [number, number] }[][] {
  const bases = themeBaseColors(theme);
  return themeToneValues().map((col) =>
    col.map(([index, tint]) => ({
      hex: hexWithTintToRgb(bases[index], tint),
      color: [index, tint],
    })),
  );
}

// Resolves a Color value to a display hex string.
export function resolveColorToHex(color: Color, theme: IronCalcTheme): string {
  if (!color) {
    return "";
  }
  if (typeof color === "string") {
    return color;
  }
  const [index, tint] = color;
  const bases = themeBaseColors(theme);
  return hexWithTintToRgb(bases[index] ?? theme.dk1, tint);
}

export const standardColors = [
  "#800000", // Dark Red
  "#FF0000", // Red
  "#FFA500", // Orange
  "#FFFF00", // Yellow
  "#92d050", // Light green
  "#00b050", // Green
  "#00b0f0", // Light blue
  "#0070c0", // Blue
  "#002060", // Dark blue
  "#7030a0", // Purple
];

// This function checks if a color is light or dark.
// This is needed to determine the icon color, as it's not visible on light colors.
export const isLightColor = (hex: string): boolean => {
  if (!hex.startsWith("#")) {
    return false;
  }

  const normalized =
    hex.length === 4
      ? `#${hex[1]}${hex[1]}${hex[2]}${hex[2]}${hex[3]}${hex[3]}`
      : hex;

  const n = parseInt(normalized.slice(1), 16);
  const r = (n >> 16) & 255;
  const g = (n >> 8) & 255;
  const b = n & 255;

  const luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
  return luminance > 160;
};

export const isWhiteColor = (color: string): boolean => {
  const upper = color.toUpperCase();
  return upper === "#FFF" || upper === "#FFFFFF";
};

export const getCheckColor = (color: string): string => {
  // --palette-common-black: #272525;
  return isLightColor(color) ? "#272525" : "#FFFFFF";
};
