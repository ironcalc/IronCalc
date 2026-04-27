---
layout: doc
outline: deep
lang: en-US
---

# Regional Settings

The **Regional Settings** allow you to change the locale and timezone for your workbook.

---

## About Locale, Timezone and Language

Locales and timezones are a **workbook** property. If you create a new workbook, it will use the default locale and timezone—not those of your previous workbook.

IronCalc allows you to have a different display language for the engine (formulas) than your locale. This means your locale might be `it-IT` but your formulas will still be in English. The language is not a property of the workbook. Two people can be looking at the same workbook in _different_ languages. This has some profound consequences. For instance the formulas `INFO` and `CELL` **always** take their parameters in English independently of the language.

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