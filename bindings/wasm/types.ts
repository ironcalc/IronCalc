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
  color: string;
  sheet_id: number;
  state: string;
}

interface CellStyleFill {
  pattern_type: string;
  fg_color?: string;
  bg_color?: string;
}

interface CellStyleFont {
  u: boolean;
  b: boolean;
  i: boolean;
  strike: boolean;
  sz: number;
  color: string;
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
  color: string;
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
  | "EndsWith";

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
  | "Signal1"
  | "Signal2"
  | "Signal3"
  | "Signal4"
  | "Signal5";

export type Cfvo =
  | "Min"
  | "Max"
  | { Number: number }
  | { Percent: number }
  | { Percentile: number }
  | { Formula: string };

export type CfRule =
  | { type: "ColorScale"; cfvo: Cfvo[]; colors: string[] }
  | { type: "CellIs"; operator: ValueOperator; formula: string; formula2: string | null; dxf_id: number }
  | { type: "Text"; operator: TextOperator; value: string; dxf_id: number }
  | { type: "TimePeriod"; time_period: PeriodType; date1: string | null; date2: string | null; dxf_id: number }
  | { type: "DuplicateValues"; dxf_id: number }
  | { type: "UniqueValues"; dxf_id: number }
  | { type: "AboveAverage"; dxf_id: number }
  | { type: "BelowAverage"; dxf_id: number }
  | { type: "Top10"; rank: number; percent: boolean; dxf_id: number }
  | { type: "Bottom10"; rank: number; percent: boolean; dxf_id: number }
  | { type: "DataBar"; cfvo: Cfvo[]; color: string; show_value: boolean }
  | { type: "IconSet"; set: IconSetType; cfvo: Cfvo[]; show_value: boolean }
  | { type: "IconSetCustom2"; set: [Icon, Icon]; cfvo: [Cfvo, Cfvo]; color: [string, string]; show_value: boolean }
  | { type: "IconSetCustom3"; set: [Icon, Icon, Icon]; cfvo: [Cfvo, Cfvo, Cfvo]; color: [string, string, string]; show_value: boolean }
  | { type: "IconSetCustom4"; set: [Icon, Icon, Icon, Icon]; cfvo: [Cfvo, Cfvo, Cfvo, Cfvo]; color: [string, string, string, string]; show_value: boolean }
  | { type: "IconSetCustom5"; set: [Icon, Icon, Icon, Icon, Icon]; cfvo: [Cfvo, Cfvo, Cfvo, Cfvo, Cfvo]; color: [string, string, string, string, string]; show_value: boolean };

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
  | "Flags3"
  | "Stars3"
  | "Quarters5"
  | "Boxes5"
  | "Ratings4"
  | "Ratings5";

export interface CfIcon {
  set: IconSetType;
  index: number;
  show_value: boolean;
}

export interface CfDataBar {
  color: string;
  value: number;
  show_value: boolean;
}

export interface CfCustomIcon {
  char: string;
  color: string;
  show_value: boolean;
}

export interface ExtendedCellStyle {
  style: CellStyle;
  icon: CfIcon | null;
  data_bar: CfDataBar | null;
  custom_icon: CfCustomIcon | null;
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
