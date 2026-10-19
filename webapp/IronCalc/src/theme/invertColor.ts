// Dark-mode color inversion.
//
// Light mode is the source of truth: every color stored in the workbook (fills,
// fonts, borders, conditional formats, sheet tabs, theme accents...) is a
// light-mode color. In dark mode we never show those colors as they are; we
// show `invertColor(color)` instead. Conversely, whatever the user picks on
// screen in dark mode goes through `uninvertColor` before it is stored, so the
// workbook keeps light-mode colors and a file looks the same whichever mode it
// was edited in.
//
// The transform works in OKLCH (perceptual lightness L, chroma C, hue H):
//
//   * The hue is preserved: red stays red, blue stays blue.
//   * The lightness is flipped and squeezed into the dark-mode range, so that
//     pure white lands exactly on the dark background and pure black on the
//     dark foreground. The distance from the background is preserved up to a
//     constant factor: a color that is "almost white" in light mode becomes
//     "almost the dark background", and so on.
//   * The chroma is preserved *relative to the sRGB gamut*: a color that is as
//     saturated as sRGB allows at its lightness stays as saturated as sRGB
//     allows at the new lightness, a pastel stays a pastel. This is what makes
//     the transform invertible: no color is ever clipped, so the inverse map
//     recovers the original (up to 8-bit rounding).
//
// Everything that is tunable lives in `DEFAULT_OPTIONS`.

export interface InvertColorOptions {
  /** OKLab lightness pure white (#FFFFFF) is mapped to: the dark background. */
  darkBackgroundL: number;
  /** OKLab lightness pure black (#000000) is mapped to: the dark foreground. */
  darkForegroundL: number;
  /**
   * Exponent applied to the distance from white (0..1) before it is mapped to
   * the distance from the dark background. 1 is linear; <1 lifts light colors
   * (pastel fills get more room above the dark background), >1 squeezes them.
   */
  lightnessGamma: number;
  /**
   * Exponent applied to the relative chroma (0..1) when going to dark mode.
   * 1 keeps the relative chroma, >1 mutes colors, <1 makes them more vivid.
   */
  chromaGamma: number;
}

// `--palette-common-white` and `--palette-sheet-default-text-color` of the
// dark theme (theme.ts). Pure white and pure black map exactly onto them.
const DARK_BACKGROUND = "#1E1E1E";
const DARK_FOREGROUND = "#E4E4E4";

export const DEFAULT_OPTIONS: InvertColorOptions = {
  darkBackgroundL: oklabLightness(DARK_BACKGROUND),
  darkForegroundL: oklabLightness(DARK_FOREGROUND),
  lightnessGamma: 1,
  chromaGamma: 1,
};

/** Light-mode color → the color shown in dark mode. */
export function invertColor(
  color: string,
  options: InvertColorOptions = DEFAULT_OPTIONS,
): string {
  return transform(color, options, false);
}

/** Color shown in dark mode → the light-mode color to store. */
export function uninvertColor(
  color: string,
  options: InvertColorOptions = DEFAULT_OPTIONS,
): string {
  return transform(color, options, true);
}

// ---------------------------------------------------------------------------
// Lightness and chroma mapping
// ---------------------------------------------------------------------------

// L (0..1, light mode) → L' (dark mode). Decreasing: white → background,
// black → foreground. The distance from white, raised to `lightnessGamma`,
// becomes the distance from the dark background.
function lightToDark(L: number, options: InvertColorOptions): number {
  const { darkBackgroundL: lo, darkForegroundL: hi, lightnessGamma } = options;
  const distance = clamp(1 - L, 0, 1) ** lightnessGamma;
  return clamp(lo + (hi - lo) * distance, 0, 1);
}

function darkToLight(L: number, options: InvertColorOptions): number {
  const { darkBackgroundL: lo, darkForegroundL: hi, lightnessGamma } = options;
  const distance = clamp((L - lo) / (hi - lo), 0, 1) ** (1 / lightnessGamma);
  return clamp(1 - distance, 0, 1);
}

function transform(
  color: string,
  options: InvertColorOptions,
  inverse: boolean,
): string {
  const parsed = parseHex(color);
  if (!parsed) {
    // Not a hex color ("", "transparent", "var(--x)", ...): nothing to invert
    return color;
  }
  const { r, g, b, alpha } = parsed;
  const key = `${inverse ? "u" : "i"}${r},${g},${b}`;
  const cache = cacheFor(options);
  const cached = cache.get(key);
  if (cached !== undefined) {
    return cached + alpha;
  }

  const [L, C, H] = toOklch(r, g, b);
  const maxC = maxChroma(L, H);
  // Relative chroma: how saturated the color is compared to what sRGB allows
  // at this lightness and hue
  let relative = maxC > CHROMA_EPSILON ? clamp(C / maxC, 0, 1) : 0;
  let newL: number;
  if (inverse) {
    newL = darkToLight(L, options);
    relative = relative ** (1 / options.chromaGamma);
  } else {
    newL = lightToDark(L, options);
    relative = relative ** options.chromaGamma;
  }
  const newC = relative * maxChroma(newL, H);
  const result = toHex(...fromOklch(newL, newC, H));

  if (cache.size > CACHE_LIMIT) {
    cache.clear();
  }
  cache.set(key, result);
  return result + alpha;
}

// One cache per options object: the sheet repeats the same few colors
// thousands of times per frame
const caches = new WeakMap<InvertColorOptions, Map<string, string>>();
const CACHE_LIMIT = 20000;

function cacheFor(options: InvertColorOptions): Map<string, string> {
  let cache = caches.get(options);
  if (!cache) {
    cache = new Map();
    caches.set(options, cache);
  }
  return cache;
}
const CHROMA_EPSILON = 1e-6;

// Largest chroma at lightness L and hue H (degrees) that is inside the sRGB
// gamut, found by bisection. sRGB is convex enough along a chroma ray for this.
function maxChroma(L: number, H: number): number {
  if (L <= 0 || L >= 1) {
    return 0;
  }
  let low = 0;
  let high = 0.4; // beyond any sRGB chroma
  for (let i = 0; i < 24; i += 1) {
    const mid = (low + high) / 2;
    if (inGamut(oklchToLinear(L, mid, H))) {
      low = mid;
    } else {
      high = mid;
    }
  }
  return low;
}

// ---------------------------------------------------------------------------
// Color space conversions: sRGB hex ↔ linear sRGB ↔ OKLab ↔ OKLCH
// (Björn Ottosson, https://bottosson.github.io/posts/oklab/)
// ---------------------------------------------------------------------------

function parseHex(
  color: string,
): { r: number; g: number; b: number; alpha: string } | null {
  if (typeof color !== "string" || !color.startsWith("#")) {
    return null;
  }
  const hex = color.slice(1);
  if (!/^[0-9a-fA-F]+$/.test(hex)) {
    return null;
  }
  let rgb: string;
  let alpha = "";
  switch (hex.length) {
    case 3:
      rgb = `${hex[0]}${hex[0]}${hex[1]}${hex[1]}${hex[2]}${hex[2]}`;
      break;
    case 4:
      rgb = `${hex[0]}${hex[0]}${hex[1]}${hex[1]}${hex[2]}${hex[2]}`;
      alpha = `${hex[3]}${hex[3]}`.toUpperCase();
      break;
    case 6:
      rgb = hex;
      break;
    case 8:
      rgb = hex.slice(0, 6);
      alpha = hex.slice(6).toUpperCase();
      break;
    default:
      return null;
  }
  const n = Number.parseInt(rgb, 16);
  return { r: (n >> 16) & 255, g: (n >> 8) & 255, b: n & 255, alpha };
}

function toHex(r: number, g: number, b: number): string {
  const channel = (v: number) =>
    clamp(Math.round(v), 0, 255).toString(16).padStart(2, "0");
  return `#${channel(r)}${channel(g)}${channel(b)}`.toUpperCase();
}

function srgbToLinear(c: number): number {
  const v = c / 255;
  return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
}

function linearToSrgb(v: number): number {
  const c = v <= 0.0031308 ? 12.92 * v : 1.055 * v ** (1 / 2.4) - 0.055;
  return c * 255;
}

function linearToOklab(
  r: number,
  g: number,
  b: number,
): [number, number, number] {
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  return [
    0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
    1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
    0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s,
  ];
}

function oklabToLinear(
  L: number,
  a: number,
  b: number,
): [number, number, number] {
  const l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const m = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const s = (L - 0.0894841775 * a - 1.291485548 * b) ** 3;
  return [
    4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
    -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
    -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s,
  ];
}

function oklchToLinear(
  L: number,
  C: number,
  H: number,
): [number, number, number] {
  const h = (H * Math.PI) / 180;
  return oklabToLinear(L, C * Math.cos(h), C * Math.sin(h));
}

function toOklch(r: number, g: number, b: number): [number, number, number] {
  const [L, a, bb] = linearToOklab(
    srgbToLinear(r),
    srgbToLinear(g),
    srgbToLinear(b),
  );
  const C = Math.hypot(a, bb);
  let H = (Math.atan2(bb, a) * 180) / Math.PI;
  if (H < 0) {
    H += 360;
  }
  return [L, C, H];
}

function fromOklch(L: number, C: number, H: number): [number, number, number] {
  const [r, g, b] = oklchToLinear(L, C, H);
  return [
    linearToSrgb(clamp(r, 0, 1)),
    linearToSrgb(clamp(g, 0, 1)),
    linearToSrgb(clamp(b, 0, 1)),
  ];
}

const GAMUT_EPSILON = 1e-5;

function inGamut([r, g, b]: [number, number, number]): boolean {
  return (
    r >= -GAMUT_EPSILON &&
    r <= 1 + GAMUT_EPSILON &&
    g >= -GAMUT_EPSILON &&
    g <= 1 + GAMUT_EPSILON &&
    b >= -GAMUT_EPSILON &&
    b <= 1 + GAMUT_EPSILON
  );
}

/** OKLab lightness (0..1) of a hex color; 0 when it is not a hex color. */
export function oklabLightness(color: string): number {
  const parsed = parseHex(color);
  if (!parsed) {
    return 0;
  }
  return toOklch(parsed.r, parsed.g, parsed.b)[0];
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
