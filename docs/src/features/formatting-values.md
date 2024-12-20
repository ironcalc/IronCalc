---
layout: doc
outline: deep
lang: en-US
---

# Formatting Values

You can format numbers in **scientific notation**, as **currencies**, **percentages**, or **dates**.

## How to Format Values

To change the format of the values:

- Click on the **€** or **%** buttons in the toolbox for quick access.
- Alternatively, click on the **123 dropdown** to select one of the predefined formats:
  - Number
  - Percentage
  - Euro (€)
  - Dollar ($)
  - British Pound (£)
  - Short Date
  - Long Date

You can also **create a custom format** directly from the dropdown.

## Creating Custom Formats

To create a custom format, you need to use a specific formatting string. Here's an example:

- **`"$"#,##0.00`**
  - **`$`**: Displays the dollar symbol. You can replace this with another currency symbol like **€** or **£**.
  - **`#,##0`**: Formats numbers with a thousands separator (`,`).
  - **`.00`**: Ensures two decimal places are always displayed.

### Common Custom Format Examples

- **`"€"#,##0.00`**: Formats numbers as Euros with two decimal places.
- **`0.00%`**: Formats numbers as percentages with two decimal places.
- **`yyyy-mm-dd`**: Displays dates in the year-month-day format.

For more advanced options, refer to standard number formatting guides.
