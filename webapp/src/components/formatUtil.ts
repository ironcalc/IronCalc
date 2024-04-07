export function increaseDecimalPlaces(numberFormat: string): string {
  // FIXME: Should it be done in the Rust? How should it work?
  // Increase decimal places for existing numbers with decimals
  const newNumberFormat = numberFormat.replace(/\.0/g, ".00");
  // If no decimal places declared, add 0.0
  if (!newNumberFormat.includes(".")) {
    if (newNumberFormat.includes("0")) {
      return newNumberFormat.replace(/0/g, "0.0");
    }
    if (newNumberFormat.includes("#")) {
      return newNumberFormat.replace(/#([^#,]|$)/g, "0.0$1");
    }
    return "0.0";
  }
  return newNumberFormat;
}

export function decreaseDecimalPlaces(numberFormat: string): string {
  // FIXME: Should it be done in the Rust? How should it work?
  // Decrease decimal places for existing numbers with decimals
  let newNumberFormat = numberFormat.replace(/\.0/g, ".");
  // Fix leftover dots
  newNumberFormat = newNumberFormat.replace(/0\.([^0]|$)/, "0$1");
  return newNumberFormat;
}

export enum NumberFormats {
  AUTO = "general",
  CURRENCY_EUR = '"€"#,##0.00',
  CURRENCY_USD = '"$"#,##0.00',
  CURRENCY_GBP = '"£"#,##0.00',
  DATE_SHORT = 'dd"/"mm"/"yyyy',
  DATE_LONG = 'dddd"," mmmm dd"," yyyy',
  PERCENTAGE = "0.00%",
  NUMBER = "#,##0.00",
}
