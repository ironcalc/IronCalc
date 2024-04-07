import { Area, Cell } from "../types";

const letters = [
  "A",
  "B",
  "C",
  "D",
  "E",
  "F",
  "G",
  "H",
  "I",
  "J",
  "K",
  "L",
  "M",
  "N",
  "O",
  "P",
  "Q",
  "R",
  "S",
  "T",
  "U",
  "V",
  "W",
  "X",
  "Y",
  "Z",
];

export function columnNameFromNumber(column: number): string {
  let columnName = "";
  let index = column;
  while (index > 0) {
    columnName = `${letters[(index - 1) % 26]}${columnName}`;
    index = Math.floor((index - 1) / 26);
  }
  return columnName;
}

// EqualTo Color Palette
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

/**
 *  Returns true if the keypress should start editing
 */
export function isEditingKey(key: string): boolean {
  if (key.length !== 1) {
    return false;
  }
  const code = key.codePointAt(0) ?? 0;
  if (code > 0 && code < 255) {
    return true;
  }
  return false;
}

export type NavigationKey =
  | "ArrowRight"
  | "ArrowLeft"
  | "ArrowDown"
  | "ArrowUp"
  | "Home"
  | "End";

export const isNavigationKey = (key: string): key is NavigationKey =>
  ["ArrowRight", "ArrowLeft", "ArrowDown", "ArrowUp", "Home", "End"].includes(
    key
  );

export const getCellAddress = (selectedArea: Area, selectedCell?: Cell) => {
  const isSingleCell =
    selectedArea.rowStart === selectedArea.rowEnd &&
    selectedArea.columnEnd === selectedArea.columnStart;

  return isSingleCell && selectedCell
    ? `${columnNameFromNumber(selectedCell.column)}${selectedCell.row}`
    : `${columnNameFromNumber(selectedArea.columnStart)}${
        selectedArea.rowStart
      }:${columnNameFromNumber(selectedArea.columnEnd)}${selectedArea.rowEnd}`;
};
