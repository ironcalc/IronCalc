---
layout: doc
outline: deep
lang: en-US
---

# NOW function

## Overview

NOW returns the current date and time as a [serial number](/features/serial-numbers.md). The integer part represents the date and the fractional part represents the time of day.

::: tip IronCalc extension
The optional `timezone` argument is an IronCalc extension. Excel, Google Sheets, and LibreOffice Calc do not support it.
:::

## Usage

### Syntax

**NOW([[<span title="Text" style="color:#E53935">timezone</span>]]) => <span title="Number" style="color:#1E88E5">serial_number</span>**

### Argument descriptions

- *timezone* ([text](/features/value-types#text), optional). An [IANA timezone](https://www.iana.org/time-zones) name (e.g. `"America/New_York"`, `"Europe/Paris"`, `"Asia/Tokyo"`). When provided, NOW returns the current time in that timezone instead of the workbook's timezone. See [Regional Settings](/features/regional-settings.md) for more details on timezones.

### Additional guidance

- When called with no argument, NOW uses the workbook's timezone setting.
- The fractional part of the returned serial number encodes the time of day: for example, `0.5` represents noon (12:00:00).
- Unlike `TODAY()`, which returns only an integer date serial, NOW returns a number with a fractional component representing the current time.
- NOW recalculates every time the workbook recalculates.

### Returned value

NOW returns a [number](/features/value-types#numbers) — a date-time [serial number](/features/serial-numbers.md) where the integer part is the date and the fractional part is the time.

### Error conditions

- If more than one argument is supplied, NOW returns the [`#ERROR!`](/features/error-types.md#error) error.
- If the `timezone` argument is not a [text](/features/value-types#text) value, NOW returns the [`#VALUE!`](/features/error-types.md#value) error.
- If the `timezone` argument is not a valid IANA timezone name, NOW returns the [`#VALUE!`](/features/error-types.md#value) error.

## Examples

| Formula | Result | Comment |
|---|---|---|
| `=NOW()` | `46000.52` | Current date and time in the workbook's timezone (example) |
| `=NOW("America/New_York")` | `46000.35` | Same moment in New York time |
| `=NOW("Asia/Tokyo")` | `46001.02` | Same moment in Tokyo time |
| `=INT(NOW())` | `46000` | Today's date serial (equivalent to `TODAY()`) |
| `=NOW()-TODAY()` | `0.52` | Fraction of the day elapsed (example: ~12:30) |

## Links

- See also IronCalc's [TODAY](/functions/date_and_time/today.md) function, which returns the current date without a time component.
- See [Regional Settings](/features/regional-settings.md) for information on workbook timezones.
- Visit Microsoft Excel's [NOW function](https://support.microsoft.com/en-us/office/now-function-3337fd29-145a-4347-b2e6-20c904739c46) page (note: Excel's NOW takes no arguments).
- Both [Google Sheets](https://support.google.com/docs/answer/3092981) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/NOW) provide versions of the NOW function (without the timezone argument).
