import type {
  CfRuleInput,
  Cfvo,
  ColorScaleThreshold,
  ConditionalFormatting,
  Dxf,
  DxfFill,
  DxfFont,
  Icon,
  IconThreshold,
  PeriodType,
  TextOperator,
  ValueOperator,
} from "@ironcalc/wasm";
import type { ColorScaleRuleData, ColorScaleStop } from "./ColorScaleRule";
import type { DataBarBound, DataBarsRuleData } from "./DataBarsRule";
import type { RuleData } from "./EditRule";
import type { FormatStyle } from "./FormatStylePicker";
import type { IconSetsRuleData, ThresholdType } from "./IconSetsRule";

// ─── Operator / period lookup tables ────────────────────────────────────────

const UI_TO_VALUE_OP: Record<string, ValueOperator> = {
  between: "Between",
  not_between: "NotBetween",
  less_than: "LessThan",
  less_than_or_equal: "LessThanOrEqual",
  greater_than: "GreaterThan",
  greater_than_or_equal: "GreaterThanOrEqual",
  equals: "Equal",
  does_not_equal: "NotEqual",
};

const VALUE_OP_TO_UI: Record<ValueOperator, string> = {
  Between: "between",
  NotBetween: "not_between",
  LessThan: "less_than",
  LessThanOrEqual: "less_than_or_equal",
  GreaterThan: "greater_than",
  GreaterThanOrEqual: "greater_than_or_equal",
  Equal: "equals",
  NotEqual: "does_not_equal",
};

const UI_TO_TEXT_OP: Record<string, TextOperator> = {
  contains: "Contains",
  does_not_contain: "DoesNotContain",
  begins_with: "BeginsWith",
  ends_with: "EndsWith",
  equals: "Equals",
};

const TEXT_OP_TO_UI: Record<TextOperator, string> = {
  Contains: "contains",
  DoesNotContain: "does_not_contain",
  BeginsWith: "begins_with",
  EndsWith: "ends_with",
  Equals: "equals",
};

const UI_TO_PERIOD: Record<string, PeriodType> = {
  between: "Between",
  not_between: "NotBetween",
  yesterday: "Yesterday",
  today: "Today",
  tomorrow: "Tomorrow",
  in_last_7_days: "Last7Days",
  in_next_7_days: "Next7Days",
  last_week: "LastWeek",
  this_week: "ThisWeek",
  next_week: "NextWeek",
  last_month: "LastMonth",
  this_month: "ThisMonth",
  next_month: "NextMonth",
  last_year: "LastYear",
  this_year: "ThisYear",
  next_year: "NextYear",
};

const PERIOD_TO_UI: Record<PeriodType, string> = {
  Between: "between",
  NotBetween: "not_between",
  Yesterday: "yesterday",
  Today: "today",
  Tomorrow: "tomorrow",
  Last7Days: "in_last_7_days",
  Next7Days: "in_next_7_days",
  LastWeek: "last_week",
  ThisWeek: "this_week",
  NextWeek: "next_week",
  LastMonth: "last_month",
  ThisMonth: "this_month",
  NextMonth: "next_month",
  LastYear: "last_year",
  ThisYear: "this_year",
  NextYear: "next_year",
};

// ─── DXF rule types ──────────────────────────────────────────────────────────

const DXF_RULE_TYPES = new Set([
  "cell_value",
  "text",
  "date",
  "formula",
  "duplicate_values",
  "unique_values",
  "blanks",
  "no_blanks",
  "errors",
  "no_errors",
]);

export function ruleTypeUsesDxf(ruleType: string): boolean {
  return DXF_RULE_TYPES.has(ruleType);
}

// ─── FormatStyle ↔ Dxf ──────────────────────────────────────────────────────

export function formatStyleToDxf(style: FormatStyle): Dxf {
  const dxf: Dxf = {};

  const hasFont =
    style.bold ||
    style.italic ||
    style.underline ||
    style.strike ||
    !!style.fontColor;
  if (hasFont) {
    const font: DxfFont = {};
    if (style.bold) font.b = true;
    if (style.italic) font.i = true;
    if (style.underline) font.u = true;
    if (style.strike) font.strike = true;
    if (style.fontColor) font.color = style.fontColor;
    dxf.font = font;
  }

  if (style.fillColor) {
    const fill: DxfFill = { pattern_type: "solid", fg_color: style.fillColor };
    dxf.fill = fill;
  }

  return dxf;
}

export function dxfToFormatStyle(dxf: Dxf | null | undefined): FormatStyle {
  return {
    bold: dxf?.font?.b ?? false,
    italic: dxf?.font?.i ?? false,
    underline: dxf?.font?.u ?? false,
    strike: dxf?.font?.strike ?? false,
    fontColor: dxf?.font?.color ?? "#000000",
    fillColor: dxf?.fill?.fg_color ?? "",
  };
}

// ─── ColorScale stop helpers ─────────────────────────────────────────────────

// "auto" for min → Min, mid → Percent(50), max → Max
function stopToCfvo(stop: ColorScaleStop): Cfvo {
  switch (stop.type) {
    case "min":
      return "Min";
    case "max":
      return "Max";
    case "number":
      return { Number: parseFloat(stop.value) };
    case "percent":
      return { Percent: parseFloat(stop.value) };
    case "percentile":
      return { Percentile: parseFloat(stop.value) };
    case "formula":
      return { Formula: stop.value };
    case "none":
      return { Percent: 50 };
  }
}

function cfvoToStop(cfvo: Cfvo, color: string): ColorScaleStop {
  if (cfvo === "Min") return { type: "min", value: "", color };
  if (cfvo === "Max") return { type: "max", value: "", color };
  if ("Number" in cfvo)
    return { type: "number", value: `${cfvo.Number}`, color };
  if ("Percent" in cfvo)
    return { type: "percent", value: `${cfvo.Percent}`, color };
  if ("Percentile" in cfvo)
    return { type: "percentile", value: `${cfvo.Percentile}`, color };
  return { type: "formula", value: cfvo.Formula, color };
}

// ─── IconSet helpers ─────────────────────────────────────────────────────────

// Preset icon arrays are ordered HIGH→LOW (index 0 = best/highest icon), matching the UI display order.
const PRESET_ICONS: Record<string, Icon[]> = {
  "dir-3-arrows-color": ["ArrowUp", "ArrowRight", "ArrowDown"],
  "dir-3-chevrons-color": ["TriangleUp", "FlatRectangle", "TriangleDown"],
  "dir-4-arrows-color": [
    "ArrowUp",
    "ArrowAngleUp",
    "ArrowAngleDown",
    "ArrowDown",
  ],
  "dir-5-arrows-color": [
    "ArrowUp",
    "ArrowAngleUp",
    "ArrowRight",
    "ArrowAngleDown",
    "ArrowDown",
  ],
  "shapes-3-circles-color": ["Circle", "Circle", "Circle"],
  "shapes-4-circles-color": ["Circle", "Circle", "Circle", "Circle"],
  "shapes-3-multiple": ["Rhombus", "Rhombus", "Rhombus"],
  "shapes-4-circles": ["Circle", "Circle", "Circle", "Circle"],
  "ind-3-checkx": ["Check", "Cross"],
  "ind-3-check-exclaim-x": ["Check", "Exclamation", "Cross"],
  "ind-4-flags": ["Flag", "Flag", "Flag"],
};

// Fallback preset by icon count when no exact match is found
const FALLBACK_PRESET_BY_COUNT: Record<number, string> = {
  2: "ind-3-checkx",
  3: "dir-3-arrows-color",
  4: "dir-4-arrows-color",
  5: "dir-5-arrows-color",
};

function findPresetIdByIcons(icons: Icon[]): string {
  const n = icons.length;
  for (const [presetId, presetIcons] of Object.entries(PRESET_ICONS)) {
    if (
      presetIcons.length === n &&
      presetIcons.every((ic, i) => ic === icons[i])
    ) {
      return presetId;
    }
  }
  return FALLBACK_PRESET_BY_COUNT[n] ?? "dir-3-arrows-color";
}

function thresholdToCfvo(th: { type: ThresholdType; value: string }): Cfvo {
  switch (th.type) {
    case "percent":
      return { Percent: parseFloat(th.value) || 0 };
    case "number":
      return { Number: parseFloat(th.value) || 0 };
    case "percentile":
      return { Percentile: parseFloat(th.value) || 0 };
    case "formula":
      return { Formula: th.value };
    case "min":
      return { Percent: 0 };
    case "max":
      return { Percent: 100 };
  }
}

function cfvoToThreshold(
  cfvo: Cfvo,
  color: string,
): { operator: "<="; value: string; type: ThresholdType; color: string } {
  if (cfvo === "Min" || cfvo === "Max")
    return {
      operator: "<=",
      value: "",
      type: cfvo === "Min" ? "min" : "max",
      color,
    };
  if ("Percent" in cfvo)
    return {
      operator: "<=",
      value: `${cfvo.Percent}`,
      type: "percent",
      color,
    };
  if ("Number" in cfvo)
    return {
      operator: "<=",
      value: `${cfvo.Number}`,
      type: "number",
      color,
    };
  if ("Percentile" in cfvo)
    return {
      operator: "<=",
      value: `${cfvo.Percentile}`,
      type: "percentile",
      color,
    };
  return { operator: "<=", value: cfvo.Formula, type: "formula", color };
}

// Build N UI threshold rows (HIGH→LOW order) from N backend cfvo/colors/iconNames/isStrict arrays
// (which are in LOW→HIGH order, index 0 = lowest icon bucket).
// Display row d corresponds to backend index j = n-1-d.
// Each UI row's value is the LOWER bound for that bucket; the last row (d=n-1) is the "else" / lowest.
function buildThresholds(
  cfvo: readonly Cfvo[],
  colors: readonly string[],
  iconNames: readonly string[],
  isStrict: readonly boolean[],
  n: number,
): IconSetsRuleData["thresholds"] {
  return Array.from({ length: n }, (_, d) => {
    const j = n - 1 - d; // backend LOW→HIGH index for display row d
    const color = colors[j] ?? "#808080";
    const iconName = iconNames[j] ?? "ArrowUp";
    const op: ">=" | ">" = (isStrict[j] ?? true) ? ">=" : ">";

    // Last display row (d = n-1, j = 0) is the lowest/"else" bucket (cfvo = Min).
    if (d === n - 1) {
      return {
        operator: op,
        value: "",
        type: "percent" as ThresholdType,
        color,
        iconName,
      };
    }

    const c = cfvo[j];
    if (!c || c === "Min") {
      return {
        operator: op,
        value: String(Math.round(((n - 1 - d) * 100) / n)),
        type: "percent" as ThresholdType,
        color,
        iconName,
      };
    }
    const base = cfvoToThreshold(c, color);
    return {
      operator: op,
      value: base.value,
      type: base.type,
      color,
      iconName,
    };
  });
}

// ─── DataBar bound helpers ───────────────────────────────────────────────────

function boundToCfvo(bound: DataBarBound): Cfvo | null {
  switch (bound.type) {
    case "automatic":
      return null;
    case "min":
      return "Min";
    case "max":
      return "Max";
    case "number":
      return { Number: parseFloat(bound.value) || 0 };
    case "percent":
      return { Percent: parseFloat(bound.value) || 0 };
    case "percentile":
      return { Percentile: parseFloat(bound.value) || 0 };
    case "formula":
      return { Formula: bound.value };
  }
}

function cfvoToBound(cfvo: Cfvo | null): DataBarBound {
  if (cfvo === null) {
    return { type: "automatic", value: "" };
  }
  if (cfvo === "Min") {
    return { type: "min", value: "" };
  }
  if (cfvo === "Max") {
    return { type: "max", value: "" };
  }
  if ("Number" in cfvo) {
    return { type: "number", value: `${cfvo.Number}` };
  }
  if ("Percent" in cfvo) {
    return { type: "percent", value: `${cfvo.Percent}` };
  }
  if ("Percentile" in cfvo) {
    return { type: "percentile", value: `${cfvo.Percentile}` };
  }
  return { type: "formula", value: cfvo.Formula };
}

// ─── Main conversion functions ───────────────────────────────────────────────

export function ruleDataToCfRule(data: RuleData): CfRuleInput | null {
  switch (data.ruleType) {
    case "color_scale": {
      if (!data.colorScale) {
        return null;
      }
      const { minimum, midpoint, maximum } = data.colorScale;
      const thresholds: ColorScaleThreshold[] =
        midpoint.type === "none"
          ? [
              { cfvo: stopToCfvo(minimum), color: minimum.color },
              { cfvo: stopToCfvo(maximum), color: maximum.color },
            ]
          : [
              { cfvo: stopToCfvo(minimum), color: minimum.color },
              { cfvo: stopToCfvo(midpoint), color: midpoint.color },
              { cfvo: stopToCfvo(maximum), color: maximum.color },
            ];
      return { type: "ColorScale", thresholds };
    }
    case "data_bars": {
      if (!data.dataBars) {
        return null;
      }
      return {
        type: "DataBar",
        min: boundToCfvo(data.dataBars.minBound),
        max: boundToCfvo(data.dataBars.maxBound),
        positive_color: data.dataBars.positiveColor,
        negative_color: data.dataBars.negativeColor,
        is_gradient: data.dataBars.gradient,
        show_value: !data.dataBars.hideCellContent,
      };
    }
    case "icon_sets": {
      if (!data.iconSets) return null;
      const { presetId, rating, thresholds, showValue } = data.iconSets;
      const sv = showValue ?? true;

      if (rating) {
        // UI thresholds are HIGH→LOW: thresholds[0]=highest star level, thresholds[n-1]=lowest/"else".
        // Backend needs LOW→HIGH: [(Percent(0), true), ...ascending lower bounds...].
        // Exclude the last (else) row, reverse the rest, map operator ">=" → is_strict=true.
        const backendThresholds: [Cfvo, boolean][] = [
          [{ Percent: 0 } as Cfvo, true],
          ...thresholds
            .slice(0, -1)
            .reverse()
            .map(
              (th) =>
                [thresholdToCfvo(th), th.operator === ">="] as [Cfvo, boolean],
            ),
        ];
        return {
          type: "IconRating",
          icon: rating.icon as Icon,
          color: rating.color,
          thresholds: backendThresholds,
          show_value: sv,
        };
      }

      const icons = PRESET_ICONS[presetId];
      if (!icons) return null;
      const n = icons.length;
      if (thresholds.length !== n) return null;
      // UI thresholds are HIGH→LOW (thresholds[0] = highest icon bucket).
      // Backend iconThresholds must be LOW→HIGH: backend index j maps to display row d = n-1-j.
      // operator ">=" → is_strict=true (bucket starts at >=)
      // operator ">"  → is_strict=false (bucket starts at >)
      const iconThresholds: IconThreshold[] = icons.map((_defaultIcon, j) => {
        const d = n - 1 - j; // display row for backend index j
        return {
          icon: (thresholds[d]?.iconName as Icon | undefined) ?? icons[d],
          cfvo: j === 0 ? ("Min" as Cfvo) : thresholdToCfvo(thresholds[d]),
          color: thresholds[d].color,
          is_strict: j === 0 ? true : thresholds[d].operator === ">=",
        };
      });
      return { type: "IconSet", thresholds: iconThresholds, show_value: sv };
    }
    case "cell_value": {
      const operator = UI_TO_VALUE_OP[data.ruleOperator];
      if (!operator) return null;
      const isBetween =
        data.ruleOperator === "between" || data.ruleOperator === "not_between";
      return {
        type: "CellIs",
        operator,
        formula: data.ruleValue,
        formula2: isBetween ? data.ruleValue2 : null,
        format: formatStyleToDxf(data.formatStyle),
      };
    }
    case "text": {
      const operator = UI_TO_TEXT_OP[data.ruleOperator];
      if (!operator) return null;
      return {
        type: "Text",
        operator,
        value: data.ruleValue,
        format: formatStyleToDxf(data.formatStyle),
      };
    }
    case "date": {
      const timePeriod = UI_TO_PERIOD[data.ruleOperator];
      if (!timePeriod) return null;
      const isBetween =
        data.ruleOperator === "between" || data.ruleOperator === "not_between";
      return {
        type: "TimePeriod",
        time_period: timePeriod,
        date1: isBetween ? data.ruleValue : null,
        date2: isBetween ? data.ruleValue2 : null,
        format: formatStyleToDxf(data.formatStyle),
      };
    }
    case "duplicate_values":
      return {
        type: "DuplicateValues",
        format: formatStyleToDxf(data.formatStyle),
      };
    case "unique_values":
      return {
        type: "UniqueValues",
        format: formatStyleToDxf(data.formatStyle),
      };
    case "blanks":
      return { type: "Blanks", format: formatStyleToDxf(data.formatStyle) };
    case "no_blanks":
      return { type: "NotBlanks", format: formatStyleToDxf(data.formatStyle) };
    case "errors":
      return { type: "Errors", format: formatStyleToDxf(data.formatStyle) };
    case "no_errors":
      return { type: "NoErrors", format: formatStyleToDxf(data.formatStyle) };
    case "formula":
      return {
        type: "Formula",
        formula: data.ruleValue,
        format: formatStyleToDxf(data.formatStyle),
      };
    default:
      return null;
  }
}

export function cfRuleToRuleData(
  cf: ConditionalFormatting,
): Partial<RuleData> | null {
  const { cf_rule } = cf;

  switch (cf_rule.type) {
    case "ColorScale": {
      const n = cf_rule.thresholds.length;
      // ColorScale must have either 2 or 3 thresholds.
      if (n < 2 || n > 3) {
        return null;
      }
      const colorScale: ColorScaleRuleData =
        n === 2
          ? {
              minimum: cfvoToStop(
                cf_rule.thresholds[0].cfvo,
                cf_rule.thresholds[0].color,
              ),
              midpoint: { type: "none", value: "", color: "" },
              maximum: cfvoToStop(
                cf_rule.thresholds[1].cfvo,
                cf_rule.thresholds[1].color,
              ),
            }
          : {
              minimum: cfvoToStop(
                cf_rule.thresholds[0].cfvo,
                cf_rule.thresholds[0].color,
              ),
              midpoint: cfvoToStop(
                cf_rule.thresholds[1].cfvo,
                cf_rule.thresholds[1].color,
              ),
              maximum: cfvoToStop(
                cf_rule.thresholds[2].cfvo,
                cf_rule.thresholds[2].color,
              ),
            };
      return {
        ruleType: "color_scale",
        ruleOperator: "",
        ruleValue: "",
        ruleValue2: "",
        colorScale,
      };
    }
    case "DataBar": {
      const dataBars: DataBarsRuleData = {
        color: cf_rule.positive_color,
        gradient: cf_rule.is_gradient,
        positiveColor: cf_rule.positive_color,
        negativeColor: cf_rule.negative_color,
        hideCellContent: !cf_rule.show_value,
        minBound: cfvoToBound(cf_rule.min || null),
        maxBound: cfvoToBound(cf_rule.max || null),
      };
      return {
        ruleType: "data_bars",
        ruleOperator: "",
        ruleValue: "",
        ruleValue2: "",
        dataBars,
      };
    }
    case "IconSet": {
      const icons = cf_rule.thresholds.map((t) => t.icon);
      const cfvo = cf_rule.thresholds.map((t) => t.cfvo);
      const colors = cf_rule.thresholds.map((t) => t.color);
      const isStrict = cf_rule.thresholds.map((t) => t.is_strict);
      const n = icons.length;
      // Backend icons are LOW→HIGH; PRESET_ICONS is HIGH→LOW — reverse for lookup.
      const presetId = findPresetIdByIcons([...icons].reverse());
      const thresholds = buildThresholds(cfvo, colors, icons, isStrict, n);
      return {
        ruleType: "icon_sets",
        ruleOperator: "",
        ruleValue: "",
        ruleValue2: "",
        iconSets: { presetId, thresholds, showValue: cf_rule.show_value },
      };
    }
    case "IconRating": {
      const n = cf_rule.thresholds.length;
      const cfvo = cf_rule.thresholds.map(([c]) => c);
      const isStrict = cf_rule.thresholds.map(([, s]) => s);
      const thresholds = buildThresholds(
        cfvo,
        Array(n).fill(cf_rule.color) as string[],
        Array(n).fill(cf_rule.icon) as string[],
        isStrict,
        n,
      );
      return {
        ruleType: "icon_sets",
        ruleOperator: "",
        ruleValue: "",
        ruleValue2: "",
        iconSets: {
          presetId: "dir-3-arrows-color",
          rating: {
            count: n as 3 | 4 | 5,
            icon: cf_rule.icon,
            color: cf_rule.color,
          },
          thresholds,
          showValue: cf_rule.show_value,
        },
      };
    }
    case "CellIs": {
      const ruleOperator = VALUE_OP_TO_UI[cf_rule.operator];
      if (!ruleOperator) return null;
      return {
        ruleType: "cell_value",
        ruleOperator,
        ruleValue: cf_rule.formula,
        ruleValue2: cf_rule.formula2 ?? "",
      };
    }
    case "Text": {
      const ruleOperator = TEXT_OP_TO_UI[cf_rule.operator];
      if (!ruleOperator) return null;
      return {
        ruleType: "text",
        ruleOperator,
        ruleValue: cf_rule.value,
        ruleValue2: "",
      };
    }
    case "TimePeriod": {
      const ruleOperator = PERIOD_TO_UI[cf_rule.time_period];
      if (!ruleOperator) return null;
      return {
        ruleType: "date",
        ruleOperator,
        ruleValue: cf_rule.date1 ?? "",
        ruleValue2: cf_rule.date2 ?? "",
      };
    }
    case "DuplicateValues":
      return {
        ruleType: "duplicate_values",
        ruleOperator: "",
        ruleValue: "",
        ruleValue2: "",
      };
    case "UniqueValues":
      return {
        ruleType: "unique_values",
        ruleOperator: "",
        ruleValue: "",
        ruleValue2: "",
      };
    case "Blanks":
      return {
        ruleType: "blanks",
        ruleOperator: "",
        ruleValue: "",
        ruleValue2: "",
      };
    case "NotBlanks":
      return {
        ruleType: "no_blanks",
        ruleOperator: "",
        ruleValue: "",
        ruleValue2: "",
      };
    case "Errors":
      return {
        ruleType: "errors",
        ruleOperator: "",
        ruleValue: "",
        ruleValue2: "",
      };
    case "NoErrors":
      return {
        ruleType: "no_errors",
        ruleOperator: "",
        ruleValue: "",
        ruleValue2: "",
      };
    case "Formula":
      return {
        ruleType: "formula",
        ruleOperator: "",
        ruleValue: cf_rule.formula,
        ruleValue2: "",
      };
    default:
      return null;
  }
}
