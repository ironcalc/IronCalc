import type { TFunction } from "i18next";

interface RuleDescriptionArgs {
  ruleType: string;
  ruleOperator: string;
  ruleValue: string;
  ruleValue2?: string;
  resolveValue?: (val: string) => string;
  t: TFunction;
}

const BETWEEN_OPERATORS = ["between", "not_between"];

const CELL_VALUE_OP_SYMBOLS: Record<string, string> = {
  less_than: "<",
  less_than_or_equal: "≤",
  greater_than: ">",
  greater_than_or_equal: "≥",
  equals: "=",
  does_not_equal: "≠",
};

const SIMPLE_RULE_TYPES = [
  "duplicate_values",
  "unique_values",
  "blanks",
  "no_blanks",
  "errors",
  "no_errors",
];

export function getRuleDescription({
  ruleType,
  ruleOperator,
  ruleValue,
  ruleValue2,
  resolveValue,
  t,
}: RuleDescriptionArgs): string {
  const resolve = resolveValue ?? ((v: string) => v);
  const v = resolve(ruleValue);
  const v2 = ruleValue2 ? resolve(ruleValue2) : ruleValue2;
  const isBetween = BETWEEN_OPERATORS.includes(ruleOperator);

  if (ruleType === "cell_value") {
    if (isBetween) {
      if (v && v2) {
        return t(`conditional_formatting.desc_cell_value_${ruleOperator}`, {
          value: v,
          value2: v2,
        });
      }
      return t("conditional_formatting.desc_cell_value", {
        op: t(`conditional_formatting.desc_op_${ruleOperator}`),
      });
    }
    const symbol = CELL_VALUE_OP_SYMBOLS[ruleOperator] ?? ruleOperator;
    return t("conditional_formatting.desc_cell_value", {
      op: v ? `${symbol} ${v}` : symbol,
    });
  }

  if (ruleType === "text") {
    const op = t(`conditional_formatting.desc_op_${ruleOperator}`, {
      defaultValue: ruleOperator,
    });
    if (v) {
      return t("conditional_formatting.desc_text_value", { op, value: v });
    }
    return t("conditional_formatting.desc_text", { op });
  }

  if (ruleType === "date") {
    if (isBetween && v && v2) {
      return t(`conditional_formatting.desc_date_${ruleOperator}`, {
        value: v,
        value2: v2,
      });
    }
    return t("conditional_formatting.desc_date", {
      op: t(`conditional_formatting.desc_op_${ruleOperator}`, {
        defaultValue: ruleOperator,
      }),
    });
  }

  if (ruleType === "formula") {
    if (v) {
      return t("conditional_formatting.desc_formula", { value: v });
    }
    return t("conditional_formatting.rule_type_formula");
  }

  if (ruleType === "color_scale") {
    return t("conditional_formatting.desc_color_scale");
  }

  if (ruleType === "data_bars") {
    return t("conditional_formatting.desc_data_bars");
  }

  if (ruleType === "icon_sets") {
    return t("conditional_formatting.icon_sets_mode_preset");
  }

  if (SIMPLE_RULE_TYPES.includes(ruleType)) {
    return t(`conditional_formatting.rule_type_${ruleType}`);
  }

  return ruleType;
}
