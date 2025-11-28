// FIXME: These two should be done in the back end and thoroughly tested
// * Dates shouldn't change
// * General depends on the value. Increase(General, 0.5) => 0.50 and so on

export function increaseDecimalPlaces(numberFormat: string): string {
  // Increase decimal places for existing numbers with decimals
  if (numberFormat === "general") {
    return "#,##0.000";
  }
  const newNumberFormat = numberFormat.replace(/\.0/g, ".00");
  // If no decimal places declared, add 0.0
  if (!newNumberFormat.includes(".")) {
    if (newNumberFormat.includes("0")) {
      return newNumberFormat.replace(/0/g, "0.0");
    }
    if (newNumberFormat.includes("#")) {
      return newNumberFormat.replace(/#([^#,]|$)/g, "0.0$1");
    }
    return numberFormat;
  }
  return newNumberFormat;
}

export function decreaseDecimalPlaces(numberFormat: string): string {
  if (numberFormat === "general") {
    return "#,##0.0";
  }
  // Decrease decimal places for existing numbers with decimals
  let newNumberFormat = numberFormat.replace(/\.0/g, ".");
  // Fix leftover dots
  newNumberFormat = newNumberFormat.replace(/0\.([^0]|$)/, "0$1");
  return newNumberFormat;
}

// FIXME: Remove all of them
export enum NumberFormats {
  AUTO = "general",
  CURRENCY_EUR = '"€"#,##0.00',
  CURRENCY_USD = '"$"#,##0.00',
  CURRENCY_GBP = '"£"#,##0.00',
  PERCENTAGE = "0.00%",
}
