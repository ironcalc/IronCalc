---
layout: doc
outline: deep
lang: en-US
---

# Financial functions

Financial functions perform common monetary calculations, such as interest and depreciation, bond pricing and yields, cash-flow analysis and the time value of money (present and future values, payments and rates).


## Glossary of financial terms

Issue date

Maturity

Settlement

Coupon

Period

Short/Long Coupon

Redemption

Par: Face value

Future value
Payment
Payment period
Principal
Schedule
Rate
Yield
discount
Treasury bill
Bond
Price

## Basis and day count convention {#basis-glossary}

A basis specifies the 'accounting days' and 'accounting days in a year' count convention to be used in a calculation. We will call it the day-count convention.

Each convention specifies two things:

1. How to calculate the number of days between two dates, date1 and date2. 
2. How to calculate the number of days in each year between two dates, date1 and date2. 

There are five day-count conventions in use in IronCalc:


0 => US 30/360
1 => Actual/Actual
2 => Actual/365
4 => European 30/360

See: YEARFRAC (Date and time)
See: https://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part2.html#Basis
See: https://en.wikipedia.org/wiki/Day_count_convention

## Frequency

1 => Annually
2 => Biannually
4 => Quarterly