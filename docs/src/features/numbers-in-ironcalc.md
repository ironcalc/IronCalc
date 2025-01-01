---
layout: doc
outline: deep
lang: en-US
---

# Numbers in IronCalc
::: warning
**Note:** This draft page is under construction 🚧

::: warning
**Note:** This page contains technical documentation

Numbers in IronCalc are [IEE 754](https://en.wikipedia.org/wiki/IEEE_754) doubles (64 bit) and are displayed uo to 15 decimal digits.

## Integers

Some Integers are well represented by IEEE 754 doubles. The largest integer that can be stored perfectly as a double is:

$$
2^53 = 9,007,199,254,740,992
$$

## Floating points

The reader should be aware that numbers like 0.1 or 0.3 are not stored perfectly by computers, _only an approximation to them_ is stored.
This results in imperfect operations like the famous `0.1 + 0.2 != 0.3`.

When comparing numbers we also compare up to 15 significant figures. With this 'trick' `=IF(0.2+0.1=0.3,TRUE,FALSE)` is actually `TRUE`.



## Compatibility issues

Excel [mostly follows IEEE 754](https://learn.microsoft.com/en-us/office/troubleshoot/excel/floating-point-arithmetic-inaccurate-result). Like IronCalc displays numbers with 15 significant digits. Excel does a few other undisclosed 'hacks'.
If the result of an addition (or subtraction) of two non very small numbers is a number close to EPS and it is the end of the calculation then it is zero.

That's is how it gets `=0.3-0.2-0.1` as `0`. However `=1*(0.3-0.2-0.1)` in Excel is `-2.77556E-17`