import type { Color } from "@ironcalc/wasm";
import { defaultThemeVariables } from "../../theme/theme";

/** Whether a `#RRGGBB` background is dark, by its relative luminance. */
export function isDarkBackground(colorHex: string): boolean {
  const hex = colorHex.replace(/^#/, "");
  if (!/^[0-9a-fA-F]{6}$/.test(hex)) {
    return false;
  }
  const [red, green, blue] = [0, 2, 4].map(
    (i) => Number.parseInt(hex.substring(i, i + 2), 16) / 255,
  );
  return 0.2126 * red + 0.7152 * green + 0.0722 * blue < 0.5;
}

/**
 * The color to draw an unfilled cell's text in. A workbook's automatic text color is
 * the theme's "Text 1" or plain black: Excel and openpyxl write `theme="1"`, Google
 * Sheets often `rgb="FF000000"`. On a dark background either one is invisible, so
 * there it follows the theme's text color instead, as Office's dark mode does. Every
 * other color is the author's choice and is kept. Filled cells keep their colors,
 * because fills don't follow the theme.
 *
 * @param color    The cell's font color, `undefined` when it has none.
 * @param resolved The color resolved to `#RRGGBB` by `model.resolveColor`.
 */
export function cellTextColor(
  color: Color,
  resolved: string,
  theme: Pick<Theme, "backgroundColor" | "defaultTextColor">,
): string {
  if (!isDarkBackground(theme.backgroundColor)) {
    return resolved;
  }
  const isText1 = Array.isArray(color) && color[0] === 1 && color[1] === 0;
  if (isText1 || resolved.toLowerCase() === "#000000") {
    return theme.defaultTextColor;
  }
  return resolved;
}

// Get a 10% transparency of an hex color
export function hexToRGBA10Percent(colorHex: string): string {
  // Remove the leading hash (#) if present
  const hex = colorHex.replace(/^#/, "");

  // Parse the hex color
  const red = Number.parseInt(hex.substring(0, 2), 16);
  const green = Number.parseInt(hex.substring(2, 4), 16);
  const blue = Number.parseInt(hex.substring(4, 6), 16);

  // Set the alpha (opacity) to 0.1 (10%)
  const alpha = 0.1;

  // Return the RGBA color string
  return `rgba(${red}, ${green}, ${blue}, ${alpha})`;
}

/**
 * Splits the given text into multiple lines. If `wrapText` is true, it applies word-wrapping
 * based on the specified canvas context, maximum width, and horizontal padding.
 *
 * - First, the text is split by newline characters so that explicit newlines are respected.
 * - If wrapping is enabled, each line is further split into words and measured against the
 *   available width. Whenever adding an extra word would exceed
 *   this limit, a new line is started.
 *
 * @param text     The text to split into lines.
 * @param wrapText Whether to apply word-wrapping or just return text split by newlines.
 * @param context  The `CanvasRenderingContext2D` used for measuring text width.
 * @param width    The maximum width for each line.
 * @returns        An array of lines (strings), each fitting within the specified width if wrapping is enabled.
 */
export function computeWrappedLines(
  text: string,
  wrapText: boolean,
  context: CanvasRenderingContext2D,
  width: number,
): string[] {
  // Split the text into lines
  const rawLines = text.split("\n");
  if (!wrapText) {
    // If there is no wrapping, return the raw lines
    return rawLines;
  }
  const wrappedLines = [];
  for (const line of rawLines) {
    const words = line.split(" ");
    let currentLine = words[0];
    for (let i = 1; i < words.length; i += 1) {
      const word = words[i];
      const testLine = `${currentLine} ${word}`;
      const textWidth = context.measureText(testLine).width;
      if (textWidth < width) {
        currentLine = testLine;
      } else {
        wrappedLines.push(currentLine);
        currentLine = word;
      }
    }
    wrappedLines.push(currentLine);
  }
  return wrappedLines;
}

function readCSSVar(name: string, style: CSSStyleDeclaration): string {
  return style.getPropertyValue(name).trim();
}

export function readThemeFromCSS(root: Element): Theme {
  const style = getComputedStyle(root);
  return {
    backgroundColor: readCSSVar("--palette-common-white", style),
    // Fills don't follow the theme, so filled cells always use the light-theme text color
    filledCellTextColor:
      defaultThemeVariables["--palette-sheet-default-text-color"],
    commonWhite: readCSSVar("--palette-common-white", style),
    gridColor: readCSSVar("--palette-sheet-grid-color", style),
    cellFontFamily: readCSSVar(
      "--palette-sheet-default-cell-font-family",
      style,
    ),
    primaryMain: readCSSVar("--palette-primary-main", style),
    headerTextColor: readCSSVar("--palette-sheet-header-text-color", style),
    headerBackground: readCSSVar("--palette-sheet-header-background", style),
    headerCornerBackground: readCSSVar(
      "--palette-sheet-header-corner-background",
      style,
    ),
    headerSelectedBackground: readCSSVar(
      "--palette-sheet-header-selected-background",
      style,
    ),
    headerBorderColor: readCSSVar("--palette-sheet-header-border-color", style),
    outlineColor: readCSSVar("--palette-sheet-outline-color", style),
    headerFont: readCSSVar("--palette-sheet-header-font", style),
    gridSeparatorColor: readCSSVar(
      "--palette-sheet-grid-separator-color",
      style,
    ),
    defaultTextColor: readCSSVar("--palette-sheet-default-text-color", style),
    headerSelectedColor: readCSSVar(
      "--palette-sheet-header-selected-color",
      style,
    ),
  };
}

export interface Theme {
  backgroundColor: string;
  gridColor: string;
  cellFontFamily: string;
  primaryMain: string;
  headerTextColor: string;
  headerBackground: string;
  headerCornerBackground: string;
  headerSelectedBackground: string;
  headerBorderColor: string;
  outlineColor: string;
  headerFont: string;
  gridSeparatorColor: string;
  defaultTextColor: string;
  filledCellTextColor: string;
  commonWhite: string;
  headerSelectedColor: string;
}
