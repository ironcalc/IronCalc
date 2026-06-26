import {
  type CellArrayStructure,
  columnNumberFromName,
  getTokens,
  type Model,
  type Range,
  type Reference,
  type TokenType,
} from "@ironcalc/wasm";
import type { JSX } from "react";
import type { ActiveRange } from "../workbookState";

function sliceString(
  text: string,
  startScalar: number,
  endScalar: number,
): string {
  const scalarValues = Array.from(text);
  const sliced = scalarValues.slice(startScalar, endScalar);
  return sliced.join("");
}

export function tokenIsReferenceType(token: TokenType): token is Reference {
  return typeof token === "object" && "Reference" in token;
}

export function tokenIsRangeType(token: TokenType): token is Range {
  return typeof token === "object" && "Range" in token;
}

function isDynamicAnchor(
  structure: CellArrayStructure,
): structure is { DynamicAnchor: [number, number] } {
  return typeof structure === "object" && "DynamicAnchor" in structure;
}

// Returns true when the cursor sits at a position where the formula grammar
// would accept a reference or range, so that arrow keys / clicking a cell can
// insert one. This asks the engine for a partial parse of the formula up to the
// cursor (see `getFormulaCompletion`).
export function isInReferenceMode(
  model: Model,
  text: string,
  cursor: number,
): boolean {
  if (!text.startsWith("=")) {
    return false;
  }
  try {
    const [sheet, row, column] = model.getSelectedCell();
    // Convert the UTF-16 cursor to a scalar offset
    const scalarCursor = Array.from(text.slice(0, cursor)).length;
    const { expecting } = model.getFormulaCompletion(
      sheet,
      row,
      column,
      text,
      scalarCursor,
    );
    return expecting.includes("Range");
  } catch (e) {
    console.error("Error in isInReferenceMode:", e);
    return false;
  }
}

// IronCalc Color Palette
export function getColor(index: number, alpha = 1): string {
  const colors = [
    {
      name: "Cyan",
      rgba: [89, 185, 188, 1],
      hex: "#59B9BC",
    },
    {
      name: "Flamingo",
      rgba: [236, 87, 83, 1],
      hex: "#EC5753",
    },
    {
      hex: "#3358B7",
      rgba: [51, 88, 183, 1],
      name: "Blue",
    },
    {
      hex: "#F8CD3C",
      rgba: [248, 205, 60, 1],
      name: "Yellow",
    },
    {
      hex: "#3BB68A",
      rgba: [59, 182, 138, 1],
      name: "Emerald",
    },
    {
      hex: "#523E93",
      rgba: [82, 62, 147, 1],
      name: "Violet",
    },
    {
      hex: "#A23C52",
      rgba: [162, 60, 82, 1],
      name: "Burgundy",
    },
    {
      hex: "#8CB354",
      rgba: [162, 60, 82, 1],
      name: "Wasabi",
    },
    {
      hex: "#D03627",
      rgba: [208, 54, 39, 1],
      name: "Red",
    },
    {
      hex: "#1B717E",
      rgba: [27, 113, 126, 1],
      name: "Teal",
    },
  ];
  if (alpha === 1) {
    return colors[index % 10].hex;
  }
  const { rgba } = colors[index % 10];
  return `rgba(${rgba[0]}, ${rgba[1]}, ${rgba[2]}, ${alpha})`;
}

// A placeholder shown when the caret sits at a position where a reference could
// be inserted (e.g. `=SUM(|`). It carries no formula text; it only hints to the
// user that arrow keys / clicking a cell will insert a reference here. The
// `ic-insert-range-hint` class is a styling hook designers can restyle later.
const referenceHint = (
  <span key="reference-hint" className="ic-insert-range-hint">
    {"  "}
  </span>
);

function getFormulaHTML(
  model: Model,
  text: string,
  cursor?: number,
): { html: JSX.Element[]; activeRanges: ActiveRange[] } {
  let html: JSX.Element[] = [];
  const activeRanges: ActiveRange[] = [];
  let colorCount = 0;
  if (text.startsWith("=")) {
    const formula = text.slice(1);
    const tokens = getTokens(formula);
    const tokenCount = tokens.length;
    const usedColors: Record<string, string> = {};
    const sheet = model.getSelectedSheet();
    const sheetList = model.getWorksheetsProperties().map((s) => s.name);

    // The reference-insertion hint is shown when the caret is in "reference
    // mode" (the grammar would accept a reference here) and no reference already
    // sits immediately after the caret. We resolve where in the rendered spans
    // it belongs by finding the first token starting at/after the caret.
    const inReferenceMode =
      cursor !== undefined && isInReferenceMode(model, text, cursor);
    // Caret as a scalar offset into `formula` (drop the leading `=`).
    const scalarCursor =
      cursor === undefined ? -1 : Array.from(text.slice(0, cursor)).length - 1;
    let hintHandled = false;

    for (let index = 0; index < tokenCount; index += 1) {
      const { token, start, end } = tokens[index];
      const isReference = tokenIsReferenceType(token);
      const isRange = tokenIsRangeType(token);

      // Insert the hint right before the first token that begins at/after the
      // caret — unless that token is itself a reference/range, in which case a
      // reference already follows the caret and no hint is needed.
      if (inReferenceMode && !hintHandled && start >= scalarCursor) {
        if (!isReference && !isRange) {
          html.push(referenceHint);
        }
        hintHandled = true;
      }

      // is next token the spill operator? If so, we want to include it in the reference
      if (
        isReference &&
        tokens[index + 1] &&
        tokens[index + 1].token === "Spill"
      ) {
        const { sheet: refSheet, row, column } = token.Reference;
        const sheetIndex = refSheet ? sheetList.indexOf(refSheet) : sheet;
        const structure = model.getCellArrayStructure(sheetIndex, row, column);
        if (isDynamicAnchor(structure)) {
          const [width, height] = structure.DynamicAnchor;
          const rowEnd = row + height - 1;
          const columnEnd = column + width - 1;
          const key = `${sheetIndex}-${row}-${column}:${rowEnd}-${columnEnd}`;
          let color = usedColors[key];
          if (!color) {
            color = getColor(colorCount);
            usedColors[key] = color;
            colorCount += 1;
          }

          // we need the whole reference A27# (so from the beginning of the reference to the end of the spill operator)
          html.push(
            <span key={index} style={{ color }}>
              {sliceString(formula, start, tokens[index + 1].end)}
            </span>,
          );

          activeRanges.push({
            sheet: sheetIndex,
            rowStart: row,
            columnStart: column,
            rowEnd,
            columnEnd,
            color,
          });
        } else {
          // If the reference is not a dynamic anchor, we treat as text
          html.push(
            <span key={index}>
              {sliceString(formula, start, tokens[index + 1].end)}
            </span>,
          );
        }
        // Skip the next token since we already processed it
        index += 1;
      } else if (isReference) {
        const { sheet: refSheet, row, column } = token.Reference;
        const sheetIndex = refSheet ? sheetList.indexOf(refSheet) : sheet;
        const key = `${sheetIndex}-${row}-${column}`;
        let color = usedColors[key];
        if (!color) {
          color = getColor(colorCount);
          usedColors[key] = color;
          colorCount += 1;
        }
        html.push(
          <span key={index} style={{ color }}>
            {sliceString(formula, start, end)}
          </span>,
        );
        activeRanges.push({
          sheet: sheetIndex,
          rowStart: row,
          columnStart: column,
          rowEnd: row,
          columnEnd: column,
          color,
        });
      } else if (isRange) {
        let {
          sheet: refSheet,
          left: { row: rowStart, column: columnStart },
          right: { row: rowEnd, column: columnEnd },
        } = token.Range;
        const sheetIndex = refSheet ? sheetList.indexOf(refSheet) : sheet;

        const key = `${sheetIndex}-${rowStart}-${columnStart}:${rowEnd}-${columnEnd}`;
        let color = usedColors[key];
        if (!color) {
          color = getColor(colorCount);
          usedColors[key] = color;
          colorCount += 1;
        }

        if (rowStart > rowEnd) {
          [rowStart, rowEnd] = [rowEnd, rowStart];
        }
        if (columnStart > columnEnd) {
          [columnStart, columnEnd] = [columnEnd, columnStart];
        }
        html.push(
          <span key={index} style={{ color }}>
            {sliceString(formula, start, end)}
          </span>,
        );

        activeRanges.push({
          sheet: sheetIndex,
          rowStart,
          columnStart,
          rowEnd,
          columnEnd,
          color,
        });
      } else {
        html.push(<span key={index}>{sliceString(formula, start, end)}</span>);
      }
    }
    // The caret sits past every token (e.g. `=SUM(`): append the hint at the end.
    if (inReferenceMode && !hintHandled) {
      html.push(referenceHint);
    }
    html = [<span key="equals">=</span>].concat(html);
  } else {
    html = [<span key="single">{text}</span>];
  }
  // Add a trailing character if text ends with newline to ensure selector's height grows
  if (text.endsWith("\n")) {
    html.push(<span key="trailing-newline">{"\n"}</span>);
  }
  return { html, activeRanges };
}

// Given a formula (without the equals sign) returns (sheetIndex, rowStart, columnStart, rowEnd, columnEnd)
// if it represent a reference or range like `Sheet1!A1` or `Sheet3!D3:D10` in an existing sheet
// If it is not a reference or range it returns null
export function parseRangeInSheet(
  model: Model,
  formula: string,
): [number, number, number, number, number] | null {
  // HACK: We are checking here the series of tokens in the range formula.
  // This is enough for our purposes but probably a more specific ranges in formula method  would be better.
  const worksheets = model.getWorksheetsProperties();
  const tokens = getTokens(formula);
  const { token } = tokens[0];
  if (tokenIsRangeType(token)) {
    const {
      sheet: refSheet,
      left: { row: rowStart, column: columnStart },
      right: { row: rowEnd, column: columnEnd },
    } = token.Range;
    if (refSheet !== null) {
      const sheetIndex = worksheets.findIndex((s) => s.name === refSheet);
      if (sheetIndex >= 0) {
        return [sheetIndex, rowStart, columnStart, rowEnd, columnEnd];
      }
    }
  } else if (tokenIsReferenceType(token)) {
    const { sheet: refSheet, row, column } = token.Reference;
    if (refSheet !== null) {
      const sheetIndex = worksheets.findIndex((s) => s.name === refSheet);
      if (sheetIndex >= 0) {
        return [sheetIndex, row, column, row, column];
      }
    }
  }
  return null;
}

// Parse a single cell like "A3" into { col, row }.
const parseCell = (cell: string): { col: number; row: number } => {
  const m = /^([A-Za-z]+)(\d+)$/.exec(cell.trim());
  if (!m) {
    throw new Error(`Invalid cell: "${cell}"`);
  }
  return { col: columnNumberFromName(m[1]), row: parseInt(m[2], 10) };
};

// Parse "A1:A4" or "A4" into a normalized rectangle.
export const parseRect = (token: string) => {
  const [a, b] = token.split(":");
  const c1 = parseCell(a);
  const c2 = b ? parseCell(b) : c1;
  return {
    minCol: Math.min(c1.col, c2.col),
    maxCol: Math.max(c1.col, c2.col),
    minRow: Math.min(c1.row, c2.row),
    maxRow: Math.max(c1.row, c2.row),
  };
};

function isRangeInRangesRaw(range: string, ranges: string): boolean {
  const target = parseRect(range);
  const rects = ranges.trim().split(/\s+/).filter(Boolean).map(parseRect);

  // A cell is covered if it lies inside at least one rectangle.
  const covered = (col: number, row: number): boolean =>
    rects.some(
      (r) =>
        col >= r.minCol &&
        col <= r.maxCol &&
        row >= r.minRow &&
        row <= r.maxRow,
    );

  // `range` is "in" `ranges` iff every cell of `range` is covered.
  for (let col = target.minCol; col <= target.maxCol; col++) {
    for (let row = target.minRow; row <= target.maxRow; row++) {
      if (!covered(col, row)) {
        return false;
      }
    }
  }
  return true;
}

/**
 * Returns true if every cell of `range` is covered by the union of `ranges`.
 *
 * @param range  A single cell or range, e.g. "A3" or "A1:A4"
 * @param ranges A space-separated list of cells/ranges, e.g. "A1:A2 A4 G6:G10"
 */
export function isRangeInRanges(range: string, ranges: string): boolean {
  try {
    return isRangeInRangesRaw(range, ranges);
  } catch (e) {
    console.error("Error in isRangeInRanges:", e);
    return false;
  }
}

/**
 * Returns true if `ranges` is a space-separated list of plain references or
 * ranges in canonical form: upper-case columns, no sheet names, no `$` absolute
 * markers, single-space separated. For example: "A1:D3 A1:E5 D3 R1:R3 S3:S5".
 */
export function isValidRanges(ranges: string): boolean {
  const cell = "[A-Z]+[1-9][0-9]*";
  const token = `${cell}(?::${cell})?`;
  return new RegExp(`^${token}(?: ${token})*$`).test(ranges);
}

export default getFormulaHTML;
