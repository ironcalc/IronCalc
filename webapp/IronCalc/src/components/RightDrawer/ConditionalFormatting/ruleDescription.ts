interface RuleDescriptionArgs {
  ruleType: string;
  ruleOperator: string;
  ruleValue: string;
  ruleValue2?: string;
  resolveValue?: (val: string) => string;
}

const BETWEEN_OPERATORS = ["between", "not_between"];

export function getRuleDescription({
  ruleType,
  ruleOperator,
  ruleValue,
  ruleValue2,
  resolveValue,
}: RuleDescriptionArgs): string {
  const resolve = resolveValue ?? ((v: string) => v);
  const v = resolve(ruleValue);
  const v2 = ruleValue2 ? resolve(ruleValue2) : ruleValue2;
  const isBetween = BETWEEN_OPERATORS.includes(ruleOperator);

  if (ruleType === "cell_value") {
    const opSymbols: Record<string, string> = {
      less_than: "<",
      less_than_or_equal: "≤",
      greater_than: ">",
      greater_than_or_equal: "≥",
      equals: "=",
      does_not_equal: "≠",
      between: "is between",
      not_between: "is not between",
    };
    const op = opSymbols[ruleOperator] ?? ruleOperator;
    if (isBetween) {
      return v && v2 ? `Cell value ${op} ${v} and ${v2}` : `Cell value ${op}`;
    }
    return v ? `Cell value ${op} ${v}` : `Cell value ${op}`;
  }

  if (ruleType === "text") {
    const opLabels: Record<string, string> = {
      contains: "contains",
      does_not_contain: "doesn't contain",
      begins_with: "starts with",
      ends_with: "ends with",
      equals: "is exactly",
    };
    const op = opLabels[ruleOperator] ?? ruleOperator;
    return v ? `Text ${op} '${v}'` : `Text ${op}`;
  }

  if (ruleType === "date") {
    const opLabels: Record<string, string> = {
      between: "is between",
      not_between: "is not between",
      yesterday: "is Yesterday",
      today: "is Today",
      tomorrow: "is Tomorrow",
      in_last_7_days: "is in Last 7 Days",
      in_next_7_days: "is in Next 7 Days",
      last_week: "is in Last Week",
      this_week: "is in This Week",
      next_week: "is in Next Week",
      last_month: "is in Last Month",
      this_month: "is in This Month",
      next_month: "is in Next Month",
      last_year: "is in Last Year",
      this_year: "is in This Year",
      next_year: "is in Next Year",
    };
    if (isBetween) {
      return v && v2
        ? `Date ${opLabels[ruleOperator]} ${v} and ${v2}`
        : `Date ${opLabels[ruleOperator] ?? ruleOperator}`;
    }
    return `Date ${opLabels[ruleOperator] ?? ruleOperator}`;
  }

  if (ruleType === "formula") {
    return v ? `Formula: ${v}` : "Formula";
  }

  if (ruleType === "color_scale") {
    return "Color Scale";
  }

  if (ruleType === "data_bars") {
    return "Data Bars";
  }

  const simpleLabels: Record<string, string> = {
    duplicate_values: "Duplicated Values",
    unique_values: "Unique Values",
    blanks: "Blanks",
    no_blanks: "No Blanks",
    errors: "Errors",
    no_errors: "No Errors",
  };
  return simpleLabels[ruleType] ?? ruleType;
}
