import {
  type Color,
  hexWithTintToRgb,
  type IronCalcTheme,
} from "@ironcalc/wasm";

function themeBaseColors(theme: IronCalcTheme): string[] {
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

// Returns [themeIndex, tint] for every cell in the themed tone grid.
// Negative tint = shade (darker), positive tint = lighter — matches the Rust Color::Theme convention.
export function computeThemeToneValues(): [number, number][][] {
  const lt = (idx: number): [number, number][] => [
    [idx, -0.05],
    [idx, -0.15],
    [idx, -0.25],
    [idx, -0.35],
    [idx, -0.5],
  ];
  const lt2 = (idx: number): [number, number][] => [
    [idx, -0.1],
    [idx, -0.25],
    [idx, -0.5],
    [idx, -0.75],
    [idx, -0.9],
  ];
  const dk = (idx: number): [number, number][] => [
    [idx, 0.5],
    [idx, 0.35],
    [idx, 0.25],
    [idx, 0.15],
    [idx, 0.05],
  ];
  const accent = (idx: number): [number, number][] => [
    [idx, 0.8],
    [idx, 0.6],
    [idx, 0.4],
    [idx, -0.25],
    [idx, -0.5],
  ];
  return [
    lt(0),
    dk(1),
    lt2(2),
    accent(3),
    accent(4),
    accent(5),
    accent(6),
    accent(7),
    accent(8),
    accent(9),
  ];
}

export function computeToneArrays(theme: IronCalcTheme): string[][] {
  const bases = themeBaseColors(theme);
  return computeThemeToneValues().map((col) =>
    col.map(([index, tint]) => hexWithTintToRgb(bases[index], tint)),
  );
}

// Resolves a Color value to a display hex string.
// Used where the model is unavailable (e.g. recent-color swatches).
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

export const staticMainColors = [
  "#FFFFFF",
  "#272525",
  "#1B717E",
  "#3BB68A",
  "#8CB354",
  "#F8CD3C",
  "#F2994A",
  "#EC5753",
  "#523E93",
  "#3358B7",
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
