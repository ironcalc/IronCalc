---
layout: doc
outline: deep
lang: en-US
---

# Serial numbers and date systems

**Note**:	For convenience, dates presented on this page are formatted in accordance with the ISO 8601 international standard. IronCalc can recognize and display dates in other formats.

IronCalc stores dates and times as positive numbers, referred to as *serial numbers*. Serial numbers can be formatted to display the date and time.

The exact way in which a serial number maps to a concrete date is called a *date system*. IronCalc supports three different date systems:

* LibreOffice/IronCalc, where the number 1 corresponds to 1899-12-31
* Excel 1900, where the number 1 corresponds to 1900-01-01.
* Excel 1904, where the number 1 corresponds to 1904-01-02.

LibreOffice/IronCalc and Excel 1900 systems coincide for serial numbers > 60. That means you can for the most part ignore date systems.
You can use the LibreOffice system with negative numbers to refer to historical events in the Gregorian calendar. For example you can find out what day of the week was Miguel de Cervantes born.

In general files imported from Excel are kept in the "Excel 1900" system and files created from IronCalc will have the LibreOffice/IronCalc system.

The explanation that follows assumes we are using the LibreOffice/IronCalc system

The integer part of a serial number represents the date, as a count of the days since the fixed starting date of 1899-12-30. Hence dates are represented by a unique, sequential integer value, for example:
* 1 corresponds to 1899-12-31.
* 2 corresponds to 1900-01-01.
* 36,526 corresponds to 2000-01-01.
* 45,658 corresponds to 2025-01-01.
* 2,958,465 corresponds to 9999-12-31.

To illustrate the concept, type the value 2 into an empty cell that is initially formatted as a number. When you subsequently change the cell to a date format, it will update to show the date 1900-01-01.

The fractional part of a serial number represents time, as a fraction of the day. For example:
* 0.0 corresponds to 00:00:00 (midnight)
* 0.041666667 corresponds to 01:00:00.
* 0.5 corresponds to 12:00:00 (noon)
* 0.75 corresponds to 18:00:00.
* 0.99 corresponds to 23:45:36. 

Since date-times are stored as numbers, they can be used for arithmetic operations in formulas. For example, it is possible to determine the difference between two dates by subtracting one serial number from the other.

## Affected functions

A large number of functions are affected by the choice of the date system

Most of the date functions:

- [DATE](/functions/date_and_time/date), [DATEVALUE](/functions/date_and_time/datevalue), [EDATE](/functions/date_and_time/edate), [EOMONTH](/functions/date_and_time/eomonth), [YEARFRAC](/functions/date_and_time/yearfrac), [DATEDIF](/functions/date_and_time/datedif), [DAYS](/functions/date_and_time/days), [DAYS360](/functions/date_and_time/days360)
- [YEAR](/functions/date_and_time/year), [MONTH](/functions/date_and_time/month), [DAY](/functions/date_and_time/day), [WEEKDAY](/functions/date_and_time/weekday), [WEEKNUM](/functions/date_and_time/weeknum), [ISOWEEKNUM](/functions/date_and_time/isoweeknum)
- [WORKDAY](/functions/date_and_time/workday), [WORKDAY.INTL](/functions/date_and_time/workday.intl), [NETWORKDAYS](/functions/date_and_time/networkdays), [NETWORKDAYS.INTL](/functions/date_and_time/networkdays.intl)
- [TODAY](/functions/date_and_time/today), [NOW](/functions/date_and_time/now)

Not affected: [TIME](/functions/date_and_time/time), [TIMEVALUE](/functions/date_and_time/timevalue), [HOUR](/functions/date_and_time/hour), [MINUTE](/functions/date_and_time/minute), [SECOND](/functions/date_and_time/second). They only touch the fractional part.

Financial functions with date arguments:

- Bonds: [PRICE](/functions/financial/price), [YIELD](/functions/financial/yield), [DURATION](/functions/financial/duration), [MDURATION](/functions/financial/mduration), [ACCRINT](/functions/financial/accrint)
- Coupons: [COUPDAYBS](/functions/financial/coupdaybs), [COUPDAYS](/functions/financial/coupdays), [COUPDAYSNC](/functions/financial/coupdaysnc), [COUPNCD](/functions/financial/coupncd), [COUPNUM](/functions/financial/coupnum), [COUPPCD](/functions/financial/couppcd)
- Odd periods: [ODDFPRICE](/functions/financial/oddfprice), [ODDFYIELD](/functions/financial/oddfyield), [ODDLPRICE](/functions/financial/oddlprice), [ODDLYIELD](/functions/financial/oddlyield)
- Treasury bills: [TBILLEQ](/functions/financial/tbilleq), [TBILLPRICE](/functions/financial/tbillprice), [TBILLYIELD](/functions/financial/tbillyield), through the one-year check
- Depreciation: [AMORLINC](/functions/financial/amorlinc), [AMORDEGRC](/functions/financial/amordegrc)

Not affected: [ACCRINTM](/functions/financial/accrintm), [DISC](/functions/financial/disc), [INTRATE](/functions/financial/intrate), [PRICEDISC](/functions/financial/pricedisc), [PRICEMAT](/functions/financial/pricemat), [RECEIVED](/functions/financial/received), [YIELDDISC](/functions/financial/yielddisc), [YIELDMAT](/functions/financial/yieldmat) and the [XIRR](/functions/financial/xirr) and [XNPV](/functions/financial/xnpv) pair. Those use only day differences, which are the same in every system.

Text and formatting:

- [TEXT](/functions/text/text), when the format code is a date format
- [VALUE](/functions/text/value), when the text parses as a date
- Cell display of any number with a date format, and typing a date into a cell, both through the formatter

Criteria matching. A criterion like ">1/3/1900" is parsed as a date through the shared criteria builder, so these are affected only when the criterion text is a date:

- [SUMIF](/functions/math_and_trigonometry/sumif), [SUMIFS](/functions/math_and_trigonometry/sumifs), [COUNTIF](/functions/statistical/countif), [COUNTIFS](/functions/statistical/countifs), [AVERAGEIF](/functions/statistical/averageif), [AVERAGEIFS](/functions/statistical/averageifs), [MINIFS](/functions/statistical/minifs), [MAXIFS](/functions/statistical/maxifs)
- [DAVERAGE](/functions/database/daverage), [DCOUNT](/functions/database/dcount), [DCOUNTA](/functions/database/dcounta), [DGET](/functions/database/dget), [DMAX](/functions/database/dmax), [DMIN](/functions/database/dmin), [DPRODUCT](/functions/database/dproduct), [DSTDEV](/functions/database/dstdev), [DSTDEVP](/functions/database/dstdevp), [DSUM](/functions/database/dsum), [DVAR](/functions/database/dvar), [DVARP](/functions/database/dvarp)

**Note**: A #VALUE! error is reported if a date-formatted cell contains an invalid serial number for that date system. For instance trying to format as date a cell that has the 3,000,000 will result in a #VALUE! error. A similar thing will happen if you use the [`TEXT`](/functions/text/text) function to format an invalid date. Functions that take serial numbers as inputs would normally return #NUM! for invalid dates

The last valid date is 9999-12-31. In the Excel 1900 and LibreOffice system that is the serial number 2,958,465. For the Excel 1904 system that is 2,957,003.
Negative numbers are only supported by the LibreOffice/IronCalc system.
In the IronCalc system the first valid date is 1582-15-10 (the Gregorian epoch). It corresponds with teh serial number -115,858. Smaller serial numbers are formatted as #VALUE!
