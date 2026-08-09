import type { IronCalcTheme } from "@ironcalc/wasm";

const ACCENT_KEYS: (keyof IronCalcTheme)[] = [
  "accent1",
  "accent2",
  "accent3",
  "accent4",
  "accent5",
  "accent6",
];

export function themeEquals(theme1: IronCalcTheme, theme2: IronCalcTheme) {
  return (
    theme1.name === theme2.name &&
    ACCENT_KEYS.every((key) => theme1[key] === theme2[key]) &&
    theme1.dk1 === theme2.dk1 &&
    theme1.lt1 === theme2.lt1 &&
    theme1.dk2 === theme2.dk2 &&
    theme1.lt2 === theme2.lt2 &&
    theme1.fol_hlink === theme2.fol_hlink &&
    theme1.hlink === theme2.hlink
  );
}
