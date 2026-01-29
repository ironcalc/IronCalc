---
layout: doc
outline: deep
lang: en-US
---

# Regional Settings

The **Regional Settings** allow you to change the locale and timezone for your workbook.

---

## About Locale and Timezone

IronCalc will try to match your browser's language and locale when possible. If your language is not supported, it will default to **English (en-US)**.

By default, the locale follows the display language unless it has been manually changed in a workbook. This means that changing the language from the **language switcher** will also update the locale in workbooks that still use the default setting.

Locales and timezones are **workbook-specific**. If you create a new workbook, it will use the default locale and timezone—not those of your previous workbook.

---

## How to Change the Locale

1. Click on the **locale indicator** in the bottom-right corner of the canvas.
   - A drawer will open on the right side.
2. Select the **locale** for your workbook.
   - This will affect how numbers, dates, and times are displayed. For instance, the number `1,234.56` in **en-GB** will appear as `1.234,56` in **es-ES**.

---

## How to Change the Timezone

By default, IronCalc uses your browser's timezone. This can be changed from the **Regional Settings** drawer.

Changing the timezone will affect date- and time-related functions, such as `TODAY()` and `NOW()`.