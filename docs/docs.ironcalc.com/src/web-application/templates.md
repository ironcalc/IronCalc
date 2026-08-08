---
layout: doc
outline: deep
lang: en-US
---

# Templates

IronCalc includes a set of ready-made templates so you can start from a working spreadsheet instead of a blank one. Each template is a regular `.xlsx` workbook you can edit freely once it's loaded.

## How to Use a Template

1. On the **Welcome screen**, click **Examples & Templates**.
   - You can also open the template gallery at any time from **File** > **New from template**.
2. In the dialog that opens, use the category pills (**All**, **Finance**, **Lifestyle**, **Project Management**, **Games**) to filter the list.
3. Click a template to instantly create a new workbook from it.

::: info
Creating a workbook from a template does not affect the original template; you get your own independent copy to edit and save.
:::

## Available Templates

### Yearly Calendar

**Category:** Lifestyle

Plan and visualize the full year ahead. By default the calendar uses the **current year**, but you can change the year cell at the top and the whole grid recalculates. It puts several IronCalc features to work:

- **Conditional formatting**: highlights today's date automatically.
- **`LAMBDA`**: builds the weekday headers and the array of days behind every month grid.
- **Themes**: switch the theme to restyle the whole calendar at once.
- **Internationalization**: change the display language and the day and month names follow.

### Crossword

**Category:** Games

Fill in the grid and solve the clues. Type one letter per white square and **conditional formatting** will check each entry automatically, turning the cell **green** when the letter is correct and **red** when it isn't, so you get instant feedback as you solve. The answer key lives on a hidden `Key` sheet.

### Travel Expenses Tracker

**Category:** Lifestyle

Track trip costs and stay on budget. Use the **Expenses Log sheet** to add your trip expenses, together with **City**, **Type**, **Date**, and **Amount**. The **Overview sheet** then gives you an expenses breakdown by category, cost per day and total, the **top 3 highest expenses**, the **top 3 days with most expenses**, and the list of **cities visited**, built with **dynamic arrays** (`UNIQUE` over a `FILTER`) so it expands on its own as you log new cities.

### Invoice

**Category:** Finance

Create client invoices. Use the **Invoice sheet** to add the list of items to include, and the **Settings sheet** to set the inputs common to every invoice: your **company details** (name, address, email, website, VAT ID), **bank details** (account holder, IBAN, BIC/SWIFT, bank name), **payment terms** (working days and VAT/tax rate), and an **invoice footer note**. Those inputs are exposed as **named ranges**, so the formulas in the Invoice sheet read them by name instead of by cell reference and are easier to follow.

### Gantt Project Tracker

**Category:** Project Management

Plan tasks and timelines on a Gantt chart. The **Settings sheet** holds the project-wide inputs: **Project Name**, **Project Start**, **Days to show**, **Today**, and up to **4 owners**. In the **Tasks sheet**, add one row per task with **Phase**, **Task Name**, **Owner**, **Start Date**, **Days**, **% Done**, and **Notes**. The **End Date** is calculated for you. The **Plan sheet** then pulls everything from these two sheets to draw the chart, entirely with **conditional formatting**: a mix of classic formula-based rules and **data bar** rules.

::: warning
Phase names are tied to **conditional formatting rules**. If you rename or add a phase, you'll need to update the matching rule for it to be colored correctly.
:::

### Weekly Timesheet

**Category:** Project Management

Log and review hours worked each week. Each day gets its own row, with columns for **Project**, **Task / Description**, **Hours**, and **Notes**; you can pick the **first day** of the range to log, and **conditional formatting** colors weekend days differently so they stand out. The **Overview sheet** totals hours by project and summarizes the hours logged across the whole time range; the list of projects it groups by is built with **dynamic arrays** (`UNIQUE` over a `FILTER`), so it fills itself in from whatever you type in the **Timesheet sheet**. You can also switch the **theme** to restyle the whole timesheet at once.


### Wordle

**Category:** Games

Guess the hidden five-letter word. **Conditional formatting** recolors the grid as you type: letters in the **right position** are marked in **green**, and letters that are in the word but in the **wrong position** are marked in **yellow**. The word **changes every day** and is picked from the long list on the `Words` sheet, using `TODAY` inside an `INDEX` over a `FILTER` so everyone gets the same word on the same date. You can edit that list or add words of your own; `REGEXTEST` validates each entry and only accepts five-letter words.

### Event Calendar

**Category:** Lifestyle

Organize and follow upcoming events. Add events to the list and they will be **highlighted in the calendars** above. Like the Yearly Calendar, it builds on several IronCalc features:

- **Conditional formatting**: marks today's date and every day that has an event on it.
- **`LAMBDA`** and **`LET`**: generate the weekday headers and the day grid of each month.
- **Themes**: switch the theme to restyle every calendar at once.
 - **Internationalization**: change the display language and the day and month names update automatically.
