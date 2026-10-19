import { describe, expect, test } from "vitest";
import {
  DEFAULT_OPTIONS,
  invertColor,
  oklabLightness,
  uninvertColor,
} from "../src/theme/invertColor";

function channels(hex: string): [number, number, number] {
  const n = Number.parseInt(hex.slice(1, 7), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

// Two hex colors are "the same" up to 8-bit rounding
function expectClose(actual: string, expected: string, tolerance = 2): void {
  const a = channels(actual);
  const e = channels(expected);
  for (let i = 0; i < 3; i += 1) {
    expect(
      Math.abs(a[i] - e[i]),
      `${actual} vs ${expected} (channel ${i})`,
    ).toBeLessThanOrEqual(tolerance);
  }
}

describe("invertColor", () => {
  test("pure white becomes the dark background, pure black the dark foreground", () => {
    expect(invertColor("#FFFFFF")).toBe("#1E1E1E");
    expect(invertColor("#000000")).toBe("#E4E4E4");
  });

  test("the dark background and foreground go back to white and black", () => {
    expect(uninvertColor("#1E1E1E")).toBe("#FFFFFF");
    expect(uninvertColor("#E4E4E4")).toBe("#000000");
  });

  test("accepts short, long and alpha hex forms and normalizes to #RRGGBB", () => {
    expect(invertColor("#fff")).toBe("#1E1E1E");
    expect(invertColor("#ffffff")).toBe("#1E1E1E");
    expect(invertColor("#FFFFFF80")).toBe("#1E1E1E80");
    expect(invertColor("#FFF8")).toBe("#1E1E1E88");
  });

  test("leaves non hex values untouched", () => {
    expect(invertColor("")).toBe("");
    expect(invertColor("transparent")).toBe("transparent");
    expect(invertColor("var(--palette-common-white)")).toBe(
      "var(--palette-common-white)",
    );
    expect(invertColor("currentColor")).toBe("currentColor");
    expect(invertColor("#12345")).toBe("#12345");
    expect(invertColor("#GGGGGG")).toBe("#GGGGGG");
  });

  test("flips lightness: light colors become dark and dark colors light", () => {
    for (const color of ["#FFF2CC", "#DDEBF7", "#C6EFCE", "#FFC7CE"]) {
      expect(oklabLightness(invertColor(color))).toBeLessThan(0.5);
    }
    for (const color of ["#800000", "#002060", "#9C0006", "#006100"]) {
      expect(oklabLightness(invertColor(color))).toBeGreaterThan(0.5);
    }
  });

  test("is monotonic in lightness", () => {
    const greys = ["#000000", "#404040", "#808080", "#BFBFBF", "#FFFFFF"];
    const inverted = greys.map((g) => oklabLightness(invertColor(g)));
    for (let i = 1; i < inverted.length; i += 1) {
      expect(inverted[i]).toBeLessThan(inverted[i - 1]);
    }
  });

  test("preserves the hue", () => {
    // red stays red, blue stays blue (and not their complementaries)
    const [r, g, b] = channels(invertColor("#FF0000"));
    expect(r).toBeGreaterThan(g + 50);
    expect(r).toBeGreaterThan(b + 50);
    const [r2, g2, b2] = channels(invertColor("#0070C0"));
    expect(b2).toBeGreaterThan(r2 + 50);
    expect(b2).toBeGreaterThan(g2);
  });

  test("never produces a grey from a saturated color", () => {
    const [r, g, b] = channels(invertColor("#FFFF00"));
    expect(Math.max(r, g, b) - Math.min(r, g, b)).toBeGreaterThan(10);
  });

  test("roundtrips through uninvertColor up to 8-bit rounding", () => {
    const colors = [
      "#FFFFFF",
      "#000000",
      "#2E414D",
      "#800000",
      "#FF0000",
      "#FFA500",
      "#92D050",
      "#00B050",
      "#00B0F0",
      "#0070C0",
      "#002060",
      "#7030A0",
      "#FFF2CC",
      "#DDEBF7",
      "#C6EFCE",
      "#FFC7CE",
      "#9C0006",
      "#006100",
      "#F2994A",
      "#808080",
      "#D9D9D9",
      "#404040",
      "#EC5753",
      "#8CB354",
    ];
    for (const color of colors) {
      expectClose(uninvertColor(invertColor(color)), color);
    }
  });

  test("roundtrips the other way round: what you pick is what you see", () => {
    const darkColors = ["#1E1E1E", "#E4E4E4", "#B70000", "#76A8FF", "#5C3900"];
    for (const color of darkColors) {
      expectClose(invertColor(uninvertColor(color)), color);
    }
  });

  test("very light saturated colors lose chroma precision but keep their hue", () => {
    // #FFFF00 → a very dark yellow; there are only a handful of 8-bit colors
    // in that corner of the gamut, so the roundtrip is only approximate
    const back = uninvertColor(invertColor("#FFFF00"));
    const [r, g, b] = channels(back);
    expect(r).toBeGreaterThan(240);
    expect(g).toBeGreaterThan(240);
    expect(b).toBeLessThan(100);
  });

  test("honours custom options", () => {
    const options = { ...DEFAULT_OPTIONS, darkBackgroundL: 0 };
    expect(invertColor("#FFFFFF", options)).toBe("#000000");
    expect(uninvertColor("#000000", options)).toBe("#FFFFFF");
    const lifted = { ...DEFAULT_OPTIONS, lightnessGamma: 0.5 };
    expect(oklabLightness(invertColor("#FFF2CC", lifted))).toBeGreaterThan(
      oklabLightness(invertColor("#FFF2CC")),
    );
    expectClose(
      uninvertColor(invertColor("#FFF2CC", lifted), lifted),
      "#FFF2CC",
    );
  });
});
