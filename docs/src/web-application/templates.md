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

Plan and visualize the full year ahead. By default the calendar uses the **current year**, but you can change the year cell at the top and the whole grid recalculates. **Conditional formatting** checks today's date and highlights it automatically, so you always know where you are at a glance.

### Mortgage Calculator

**Category:** Finance

Estimate payments, interest, and overall cost. The first sheet, **Settings**, is the calculator itself: enter the **home price**, **down payment**, **interest rate**, **loan term**, and **start date**, and it gives you the **loan amount**, **number of payments**, **total monthly payment**, **total amount paid**, **total interest paid**, payments made so far, the remaining balance, and the **payoff date**. The **Amortization Schedule sheet** then lists the month-by-month breakdown of every payment, with the principal/interest split and running balance for each one.

### Crossword

**Category:** Games

Fill in the grid and solve the clues. Type one letter per white square—**conditional formatting** checks each entry automatically, turning the cell **green** when the letter is correct and **red** when it isn't, so you get instant feedback as you solve. The answer key lives on a hidden `Key` sheet.

### Travel Expenses Tracker

**Category:** Lifestyle

Track trip costs and stay on budget. Use the **Expenses Log sheet** to add your trip expenses, together with **City**, **Type**, **Date**, and **Amount**. The **Overview sheet** then gives you an expenses breakdown by category, cost per day and total, the **top 3 highest expenses**, the **top 3 days with most expenses**, and the list of **cities visited**.

### Invoice

**Category:** Finance

Create client invoices. Use the **Invoice sheet** to add the list of items to include, and the **Settings sheet** to set the inputs common to every invoice: your **company details** (name, address, email, website, VAT ID), **bank details** (account holder, IBAN, BIC/SWIFT, bank name), **payment terms** (working days and VAT/tax rate), and an **invoice footer note**.

### Gantt Project Tracker

**Category:** Project Management

Plan tasks and timelines on a Gantt chart. The **Settings sheet** holds the project-wide inputs: **Project Name**, **Project Start**, **Days to show**, **Today**, and up to **4 owners**. In the **Tasks sheet**, add one row per task with **Phase**, **Task Name**, **Owner**, **Start Date**, **Days**, **% Done**, and **Notes**—the **End Date** is calculated for you. The **Plan sheet** then pulls everything from these two sheets to draw the chart.

::: warning
Phase names are tied to **conditional formatting rules**. If you rename or add a phase, you'll need to update the matching rule for it to be colored correctly.
:::

### Weekly Timesheet

**Category:** Project Management

Log and review hours worked each week. Each day gets its own row, with columns for **Project**, **Task / Description**, **Hours**, and **Notes**; you can pick the **first day** of the range to log. The **Overview sheet** totals hours by project and summarizes the hours logged across the whole time range.

### EU Salary Calculator

**Category:** Finance

Estimate net salary after taxes in **Germany 🇩🇪, France 🇫🇷, Spain 🇪🇸, Italy 🇮🇹, the Netherlands 🇳🇱, Belgium 🇧🇪, Austria 🇦🇹, and Poland 🇵🇱**. Enter the **country**, the **annual gross salary**, and whether **Church Tax** applies (Germany only), and the sheet breaks the result down into five sections: gross salary, employee social security contributions, income tax, net salary, and employer cost—including the effective tax rate and the total cost to the employer.

::: info
Results are **orientative only**, meant for quick comparisons rather than exact payroll figures.
:::

### Absence Schedule

**Category:** Project Management

Track team vacations and time off for up to **five employees**, configured in the **Settings sheet**. Besides vacation, you can log **sick leave**, **personal time off**, and up to **2 custom categories** of your own. The sheet covers a single month, but you can adapt it to any month or **duplicate the sheet** to cover the full year. An overview on the right side totals the number of days logged per category.

### Wordle

**Category:** Games

Guess the hidden five-letter word. Letters in the **right position** are marked in **green**; letters that are in the word but in the **wrong position** are marked in **yellow**. The word **changes every day**.

### Event Calendar

**Category:** Lifestyle

Organize and follow upcoming events. Add events to the list and they will be **highlighted in the calendars** above.
