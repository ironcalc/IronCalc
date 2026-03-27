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

function readCSSVar(name: string, el: Element): string {
  return getComputedStyle(el).getPropertyValue(name).trim();
}

function readNumberCSSVar(name: string, el: Element): number {
  const value = readCSSVar(name, el);
  return parseFloat(value);
}

export function readThemeFromCSS(root: Element): Theme {
  return {
    backgroundColor: readCSSVar("--palette-common-white", root),
    commonWhite: readCSSVar("--palette-common-white", root),
    gridColor: readCSSVar("--palette-sheet-grid-color", root),
    cellFontFamily: readCSSVar(
      "--palette-sheet-default-cell-font-family",
      root,
    ),
    primaryMain: readCSSVar("--palette-primary-main", root),
    headerTextColor: readCSSVar("--palette-sheet-header-text-color", root),
    headerBackground: readCSSVar("--palette-sheet-header-background", root),
    headerSelectedBackground: readCSSVar(
      "--palette-sheet-header-selected-background",
      root,
    ),
    headerBorderColor: readCSSVar("--palette-sheet-header-border-color", root),
    outlineColor: readCSSVar("--palette-sheet-outline-color", root),
    headerFontFamily: readCSSVar("--palette-sheet-header-font-family", root),
    headerFontSize: readNumberCSSVar("--palette-sheet-header-font-size", root),
    gridSeparatorColor: readCSSVar(
      "--palette-sheet-grid-separator-color",
      root,
    ),
    defaultTextColor: readCSSVar("--palette-sheet-default-text-color", root),
    headerSelectedColor: readCSSVar(
      "--palette-sheet-header-selected-color",
      root,
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
  headerSelectedBackground: string;
  headerBorderColor: string;
  outlineColor: string;
  headerFontFamily: string;
  headerFontSize: number;
  gridSeparatorColor: string;
  defaultTextColor: string;
  commonWhite: string;
  headerSelectedColor: string;
}
