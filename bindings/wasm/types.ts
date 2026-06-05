export interface Area {
  sheet: number;
  row: number;
  column: number;
  width: number;
  height: number;
}

export enum BorderType {
  All = "All",
  Inner = "Inner",
  Outer = "Outer",
  Top = "Top",
  Right = "Right",
  Bottom = "Bottom",
  Left = "Left",
  CenterH = "CenterH",
  CenterV = "CenterV",
  None = "None",
}

export interface BorderArea {
  item: BorderItem;
  type: BorderType;
}

type ErrorType =
  | "REF"
  | "NAME"
  | "VALUE"
  | "DIV"
  | "NA"
  | "NUM"
  | "ERROR"
  | "NIMPL"
  | "SPILL"
  | "CALC"
  | "CIRC";

type OpCompareType =
  | "LessThan"
  | "GreaterThan"
  | "Equal"
  | "LessOrEqualThan"
  | "GreaterOrEqualThan"
  | "NonEqual";

type OpSumType = "Add" | "Minus";

type OpProductType = "Times" | "Divide";

interface ReferenceType {
  sheet: string | null;
  row: number;
  column: number;
  absolute_column: boolean;
  absolute_row: boolean;
}

interface ParsedReferenceType {
  column: number;
  row: number;
  absolute_column: boolean;
  absolute_row: boolean;
}

interface Reference {
  Reference: ReferenceType;
}

interface Range {
  Range: {
    sheet: string | null;
    left: ParsedReferenceType;
    right: ParsedReferenceType;
  };
}

export type TokenType =
  | "Illegal"
  | "Eof"
  | { Ident: string }
  | { String: string }
  | { Boolean: boolean }
  | { Number: number }
  | { ERROR: ErrorType }
  | { COMPARE: OpCompareType }
  | { SUM: OpSumType }
  | { PRODUCT: OpProductType }
  | "POWER"
  | "LPAREN"
  | "RPAREN"
  | "COLON"
  | "SEMICOLON"
  | "LBRACKET"
  | "RBRACKET"
  | "LBRACE"
  | "RBRACE"
  | "COMMA"
  | "BANG"
  | "PERCENT"
  | "AND"
  | Reference
  | Range;

export interface MarkedToken {
  token: TokenType;
  start: number;
  end: number;
}

export type CellArrayStructure =
  | "SingleCell"
  | { DynamicChild: [number, number, number, number] }
  | { DynamicAnchor: [number, number] }
  | { ArrayAnchor: [number, number] }
  | { ArrayChild: [number, number, number, number] };

export interface WorksheetProperties {
  name: string;
  /** Tab color. Absent when Color::None. */
  color?: Color;
  sheet_id: number;
  state: string;
}

/**
 * A cell color value. Matches the Rust `Color` enum serialized with `#[serde(untagged)]`:
 * - `string`           → `Color::Rgb("#RRGGBB")`
 * - `[number, number]` → `Color::Theme(index, tint)`
 * - absent/undefined   → `Color::None` (field omitted via skip_serializing_if)
 *
 * Pass to `model.resolveColor(color)` to get the final CSS hex string.
 */
export type Color = string | [number, number];

interface CellStyleFill {
  color?: Color;
}

interface CellStyleFont {
  u: boolean;
  b: boolean;
  i: boolean;
  strike: boolean;
  sz: number;
  color?: Color;
  name: string;
  family: number;
  scheme: string;
}

export interface BorderOptions {
  color: string;
  style: BorderStyle;
  border: BorderType;
}

export enum BorderStyle {
  Thin = "thin",
  Medium = "medium",
  Thick = "thick",
  Double = "double",
  Dotted = "dotted",
  SlantDashDot = "slantdashdot",
  MediumDashed = "mediumdashed",
  MediumDashDotDot = "mediumdashdotdot",
  MediumDashDot = "mediumdashdot",
}

interface BorderItem {
  style: string;
  color?: Color;
}

interface CellStyleBorder {
  diagonal_up?: boolean;
  diagonal_down?: boolean;
  left: BorderItem;
  right: BorderItem;
  top: BorderItem;
  bottom: BorderItem;
  diagonal: BorderItem;
}

export type VerticalAlignment =
  | "bottom"
  | "center"
  | "distributed"
  | "justify"
  | "top";

export type HorizontalAlignment =
  | "left"
  | "center"
  | "right"
  | "general"
  | "centerContinuous"
  | "distributed"
  | "fill"
  | "justify";

interface Alignment {
  horizontal: HorizontalAlignment;
  vertical: VerticalAlignment;
  wrap_text: boolean;
}

export interface CellStyle {
  read_only: boolean;
  quote_prefix: boolean;
  fill: CellStyleFill;
  font: CellStyleFont;
  border: CellStyleBorder;
  num_fmt: string;
  alignment?: Alignment;
}

export type ValueOperator =
  | "Equal"
  | "GreaterThan"
  | "GreaterThanOrEqual"
  | "LessThan"
  | "LessThanOrEqual"
  | "NotEqual"
  | "Between"
  | "NotBetween";

export type TextOperator =
  | "Contains"
  | "DoesNotContain"
  | "BeginsWith"
  | "EndsWith"
  | "Equals";

export type PeriodType =
  | "Between"
  | "NotBetween"
  | "Yesterday"
  | "Today"
  | "Tomorrow"
  | "Last7Days"
  | "Next7Days"
  | "LastWeek"
  | "ThisWeek"
  | "NextWeek"
  | "LastMonth"
  | "ThisMonth"
  | "NextMonth"
  | "LastYear"
  | "ThisYear"
  | "NextYear";

export type Icon =
  | "ArrowUp"
  | "ArrowRight"
  | "ArrowDown"
  | "ArrowAngleUp"
  | "ArrowAngleDown"
  | "Circle"
  | "TriangleUp"
  | "TriangleDown"
  | "FlatRectangle"
  | "Rhombus"
  | "Flag"
  | "Check"
  | "Cross"
  | "Exclamation"
  | "Star"
  | "Heart"
  | "ThumbsUp"
  | "ThumbsDown"
  | "TriangleUpFilled"
  | "TriangleDownFilled";

export type Cfvo =
  | "Min"
  | "Max"
  | { Number: number }
  | { Percent: number }
  | { Percentile: number }
  | { Formula: string };

export interface ColorScaleThreshold {
  cfvo: Cfvo;
  color: string;
}

export interface IconThreshold {
  icon: Icon;
  cfvo: Cfvo;
  color: string;
  is_strict: boolean;
}

/** Stored CF rule returned by getConditionalFormattingList (no format field — use getDxfForConditionalFormatting to retrieve it). */
export type CfRule =
  | { type: "ColorScale"; thresholds: ColorScaleThreshold[] }
  | { type: "CellIs"; operator: ValueOperator; formula: string; formula2: string | null; stop_if_true: boolean }
  | { type: "Formula"; formula: string; stop_if_true: boolean }
  | { type: "Text"; operator: TextOperator; value: string; stop_if_true: boolean }
  | { type: "TimePeriod"; time_period: PeriodType; date1: string | null; date2: string | null; stop_if_true: boolean }
  | { type: "DuplicateValues"; stop_if_true: boolean }
  | { type: "UniqueValues"; stop_if_true: boolean }
  | { type: "Blanks"; stop_if_true: boolean }
  | { type: "NotBlanks"; stop_if_true: boolean }
  | { type: "Errors"; stop_if_true: boolean }
  | { type: "NoErrors"; stop_if_true: boolean }
  | { type: "AboveAverage"; stop_if_true: boolean }
  | { type: "BelowAverage"; stop_if_true: boolean }
  | { type: "Top10"; rank: number; percent: boolean; stop_if_true: boolean }
  | { type: "Bottom10"; rank: number; percent: boolean; stop_if_true: boolean }
  | { type: "DataBar"; min: Cfvo | null; max: Cfvo | null; positive_color: string; negative_color: string; is_gradient: boolean; show_value: boolean }
  | { type: "IconSet"; thresholds: IconThreshold[]; show_value: boolean }
  | { type: "IconRating"; icon: Icon; color: string; thresholds: [Cfvo, boolean][]; show_value: boolean };

/** Input CF rule for addConditionalFormatting / updateConditionalFormatting.
 *  Dxf-based variants carry an inline `format` and a `stop_if_true` flag. */
export type CfRuleInput =
  | { type: "ColorScale"; thresholds: ColorScaleThreshold[] }
  | { type: "CellIs"; operator: ValueOperator; formula: string; formula2: string | null; format: Dxf; stop_if_true: boolean }
  | { type: "Formula"; formula: string; format: Dxf; stop_if_true: boolean }
  | { type: "Text"; operator: TextOperator; value: string; format: Dxf; stop_if_true: boolean }
  | { type: "TimePeriod"; time_period: PeriodType; date1: string | null; date2: string | null; format: Dxf; stop_if_true: boolean }
  | { type: "DuplicateValues"; format: Dxf; stop_if_true: boolean }
  | { type: "UniqueValues"; format: Dxf; stop_if_true: boolean }
  | { type: "Blanks"; format: Dxf; stop_if_true: boolean }
  | { type: "NotBlanks"; format: Dxf; stop_if_true: boolean }
  | { type: "Errors"; format: Dxf; stop_if_true: boolean }
  | { type: "NoErrors"; format: Dxf; stop_if_true: boolean }
  | { type: "AboveAverage"; format: Dxf; stop_if_true: boolean }
  | { type: "BelowAverage"; format: Dxf; stop_if_true: boolean }
  | { type: "Top10"; rank: number; percent: boolean; format: Dxf; stop_if_true: boolean }
  | { type: "Bottom10"; rank: number; percent: boolean; format: Dxf; stop_if_true: boolean }
  | { type: "DataBar"; min: Cfvo | null; max: Cfvo | null; positive_color: string; negative_color: string; is_gradient: boolean; show_value: boolean }
  | { type: "IconSet"; thresholds: IconThreshold[]; show_value: boolean }
  | { type: "IconRating"; icon: Icon; color: string; thresholds: [Cfvo, boolean][]; show_value: boolean };

export type FontScheme = "minor" | "major" | "none";

export interface DxfFont {
  strike?: boolean;
  u?: boolean;
  b?: boolean;
  i?: boolean;
  sz?: number;
  color?: Color;
  name?: string;
  family?: number;
  scheme?: FontScheme;
}

export interface DxfFill {
  color?: Color;
}

export interface DxfBorderItem {
  style: BorderStyle;
  color?: Color;
}

export interface DxfBorder {
  diagonal_up?: boolean;
  diagonal_down?: boolean;
  left?: DxfBorderItem;
  right?: DxfBorderItem;
  top?: DxfBorderItem;
  bottom?: DxfBorderItem;
  diagonal?: DxfBorderItem;
}

export interface DxfNumFmt {
  num_fmt_id: number;
  format_code: string;
}

export interface DxfAlignment {
  horizontal?: HorizontalAlignment;
  vertical?: VerticalAlignment;
  wrap_text?: boolean;
}

export interface Dxf {
  font?: DxfFont;
  fill?: DxfFill;
  border?: DxfBorder;
  num_fmt?: DxfNumFmt;
  alignment?: DxfAlignment;
}

export interface ConditionalFormatting {
  range: string;
  cf_rule: CfRule;
  priority: number;
}

export type IconSetType =
  | "Arrows3"
  | "ArrowsGray3"
  | "Arrows4"
  | "ArrowsGray4"
  | "Arrows5"
  | "ArrowsGray5"
  | "Triangles3"
  | "TrafficLights3"
  | "TrafficLights3Rimmed"
  | "TrafficLights4"
  | "Signs3"
  | "RedToBlack4"
  | "Symbols3Circled"
  | "Symbols3Uncircled"
  | "Flags3";

export interface CfIcon {
  icon: Icon;
  color: string;
  show_value: boolean;
}

export interface CfDataBar {
  positive_color: string;
  negative_color: string;
  is_gradient: boolean;
  value: number;
  axis_position: number;
  show_value: boolean;
}

export interface CfRating {
  icon: Icon;
  count: number;
  max: number;
  color: string;
  show_value: boolean;
}

export interface ExtendedCellStyle {
  style: CellStyle;
  icon: CfIcon | null;
  data_bar: CfDataBar | null;
  rating: CfRating | null;
}

export interface SelectedView {
  sheet: number;
  row: number;
  column: number;
  range: [number, number, number, number];
  top_row: number;
  left_column: number;
}

// type ClipboardData = {
//   [row: number]: {
//       [column: number]: ClipboardCell;
//   };
// };

// type ClipboardData = Record<string, Record <string, ClipboardCell>>;
type ClipboardData = Map<number, Map <number, ClipboardCell>>;

export interface ClipboardCell {
  text: string;
  style: CellStyle;
}

export interface Clipboard {
  csv: string;
  data: ClipboardData;
  range: [number, number, number, number];
}

export interface DefinedName {
  name: string;
  scope?: number;
  formula: string;
}

export interface FmtSettings {
  currency: string;
  currency_format: string;
  short_date: string;
  short_date_example: string;
  long_date: string;
  long_date_example: string;
  number_fmt: string;
  number_example: string;
}

/** A named cell style (e.g. "Normal", "Heading 1", or a custom style). */
export interface NamedStyle {
  /** The style name. */
  name: string;
  /** The full style definition. */
  style: CellStyle;
}

/** A builtin workbook color theme returned by `getThemeList`. */
export interface IronCalcTheme {
  /** Display name, e.g. "Office", "Retrospect". */
  name: string;
  /** Dark 1 (text/background). */
  dk1: string;
  /** Light 1 (text/background). */
  lt1: string;
  /** Dark 2 (text/background). */
  dk2: string;
  /** Light 2 (text/background). */
  lt2: string;
  accent1: string;
  accent2: string;
  accent3: string;
  accent4: string;
  accent5: string;
  accent6: string;
  /** Hyperlink color. */
  hlink: string;
  /** Followed-hyperlink color. */
  fol_hlink: string;
}
