export const mainColors = [
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

const lightTones = [
  "#F5F5F5", // --palette-grey-50
  "#F2F2F2", // --palette-grey-100
  "#EEEEEE", // --palette-grey-200
  "#E0E0E0", // --palette-grey-300
  "#BDBDBD", // --palette-grey-400
];

const darkTones = [
  "#9E9E9E", // --palette-grey-500
  "#757575", // --palette-grey-600
  "#616161", // --palette-grey-700
  "#424242", // --palette-grey-800
  "#333333", // --palette-grey-900
];

const tealTones = ["#BBD4D8", "#82B1B8", "#498D98", "#1E5A63", "#224348"];
const greenTones = ["#C4E9DC", "#93D7BF", "#62C5A1", "#358A6C", "#2F5F4D"];
const limeTones = ["#DDE8CC", "#C0D5A1", "#A3C276", "#6E8846", "#4F5E38"];
const yellowTones = ["#FDF0C5", "#FBE394", "#F9D764", "#B99A36", "#7A682E"];
const orangeTones = ["#FBE0C9", "#F8C79B", "#F5AD6E", "#B5763F", "#785334"];
const redTones = ["#F9CDCB", "#F5A3A0", "#F07975", "#B14845", "#763937"];
const purpleTones = ["#CBC5DF", "#A095C4", "#7565A9", "#453672", "#382F51"];
const blueTones = ["#C2CDE9", "#8FA3D7", "#5D79C5", "#30498B", "#2C395F"];

export const toneArrays = [
  lightTones,
  darkTones,
  tealTones,
  greenTones,
  limeTones,
  yellowTones,
  orangeTones,
  redTones,
  purpleTones,
  blueTones,
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
