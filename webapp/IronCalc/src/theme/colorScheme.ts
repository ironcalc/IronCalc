import type { Color, IronCalcTheme } from "@ironcalc/wasm";
import { createContext, useCallback, useContext } from "react";
import { resolveColorToHex } from "../components/ColorPicker/util";
import {
  DEFAULT_OPTIONS,
  type InvertColorOptions,
  invertColor,
  uninvertColor,
} from "./invertColor";

export type ColorScheme = "light" | "dark";

// The color scheme IronCalc is rendered in. Set by the `colorScheme` prop of
// `IronCalc`; "light" when the widget is used without it.
export const ColorSchemeContext = createContext<ColorScheme>("light");

export function useColorScheme(): ColorScheme {
  return useContext(ColorSchemeContext);
}

// The knobs of the dark-mode inversion (invertColor.ts). `IronCalc` owns them
// so they can be tuned at runtime from the regional settings drawer.
export interface InvertColorOptionsState {
  options: InvertColorOptions;
  setOptions: (options: InvertColorOptions) => void;
}

export const InvertColorOptionsContext = createContext<InvertColorOptionsState>(
  { options: DEFAULT_OPTIONS, setOptions: () => {} },
);

export function useInvertColorOptions(): InvertColorOptionsState {
  return useContext(InvertColorOptionsContext);
}

// Workbook colors are light-mode colors. These two functions map between what
// is stored in the workbook and what is shown on screen (see invertColor.ts).
// Non hex values ("", "transparent", "var(--x)") pass through untouched.

/** Stored (light-mode) color → color shown on screen. */
export function toDisplayHex(
  hex: string,
  scheme: ColorScheme,
  options: InvertColorOptions = DEFAULT_OPTIONS,
): string {
  return scheme === "dark" ? invertColor(hex, options) : hex;
}

/** Color picked on screen → color to store in the workbook. */
export function fromDisplayHex(
  hex: string,
  scheme: ColorScheme,
  options: InvertColorOptions = DEFAULT_OPTIONS,
): string {
  return scheme === "dark" ? uninvertColor(hex, options) : hex;
}

/** `toDisplayHex` bound to the current color scheme and options. */
export function useDisplayHex(): (hex: string) => string {
  const scheme = useColorScheme();
  const { options } = useInvertColorOptions();
  return useCallback(
    (hex: string) => toDisplayHex(hex, scheme, options),
    [scheme, options],
  );
}

/** `fromDisplayHex` bound to the current color scheme and options. */
export function useFromDisplayHex(): (hex: string) => string {
  const scheme = useColorScheme();
  const { options } = useInvertColorOptions();
  return useCallback(
    (hex: string) => fromDisplayHex(hex, scheme, options),
    [scheme, options],
  );
}

/**
 * Resolves a workbook `Color` (hex, theme reference or none) against the
 * workbook theme and maps it to the color shown on screen in the current
 * color scheme. Returns "" for an absent color.
 */
export function useResolveColor(
  theme: IronCalcTheme,
): (color: Color) => string {
  const scheme = useColorScheme();
  const { options } = useInvertColorOptions();
  return useCallback(
    (color: Color) =>
      toDisplayHex(resolveColorToHex(color, theme), scheme, options),
    [theme, scheme, options],
  );
}
