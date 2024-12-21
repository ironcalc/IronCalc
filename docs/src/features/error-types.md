---
layout: doc
outline: deep
lang: en-US
---

# Error Types

::: warning
**Note:** This draft page is under construction 🚧
:::

The result of a formula is sometimes an _error_. In some situations those errors are expected and your formulas might be dealing with them.
The error `#N/A` might signal that there is no data to evaluate the formula yet. Maybe the payroll has not been introduced for that month just yet.

Some other errors like `#SPILL!`, `#CIRC!` or `#ERROR!` signal an error in your spreadsheet logic and must be corrected.

The first kind of errors or 'common errors' are found in other spreadsheet engines like Excel while other errors like `#ERROR!` or `#N/IMPL` are particular to IronCalc.

## Common Errors

### **`#VALUE!`**

It might be caused by mismatched data types (e.g., text used where numbers are expected):

```
=5+"two"
```

The engine doesn't know how to add the number `5` to the string `two` resulting in a `#VALUE!`.

It is an actual error in your spreadsheet. It indicates that the formula isn’t working as intended.

### **`#DIV/0!`**

Division by zero or an empty cell:

```
=1/0
```

Usually this is an error. However, in cases where a denominator might be blank (e.g., data not yet filled in), this could be expected. Use `IFERROR` or `IF` to handle it:

```
=IF(B1=0, "N/A", A1/B1)
```

### **`#NAME?`**

Found when a name is not recognized. Maybe a misspelled name for a function or a reference to a previously defined name that has since been deleted:

```
=UNKNOWN_FUNCTION(A1)
```

This indicates an error in your spreadsheet logic.

### **`#REF!`**

Indicates an invalid cell reference, often from deleting cells used in a formula.

They can appear as a result of a computation or in a formula. Example:

```
=Sheet34!A1
```

If `Sheet34` doesn't exist it will return `#REF!`

This is a genuine error. It indicates that part of your formula references a cell or range that is missing.

### **`#NUM!`**

Invalid numeric operation (e.g., calculating the square root of a negative number).  
Adjust the formula to ensure valid numeric operations.

Sometimes a `#NUM!` error might be expected, signalling to the user that some parameter is out of scope.

### **`#N/A`**

A value is not available, often in lookup functions like VLOOKUP.

This is frequently not an error in your spreadsheet logic.

You can produce a prettier answer using the [`IFNA`](/functions/information/isna) formula:

```
=IFNA(VLOOKUP(A1, B1:C10, 2, FALSE), "Not Found")
```

### **`#NULL!`**

Incorrect range operator in a formula (e.g., missing a colon between cell references).

### **`#SPILL!`**

A cell in a formula will overwrite content in other cells.
This cannot happen right now in IronCalc as formulas don't spill yet.

### **`#CIRC!`**

Circular reference. This is an error in your spreadsheet and must be fixed.
It means that during the course of a computation, a circular dependency was found.

A circular dependency is a dependency of a formula on itself.

For instance, in the cell `A1` the formula `=A1*2` is a circular dependency.

Other spreadsheet engines use circular dependencies to do "loop computations", run "sensitivity analysis" or "goal seek".

IronCalc doesn't support any of those at the moment.

## IronCalc specific errors

### **`#ERROR!`**

General formula issue, like syntax errors or invalid references.
In general, Excel does not let you enter incorrect formulas, but IronCalc will.

This will make your workbook imcompatible with Excel.

Typical examples might be an incomplete formula, such as `=A1+`, or a function call with too few arguments, such as `=FV(1,2)`.

### **`#N/IMPL!`**

A particular feature is not yet implemented in IronCalc

Check if there is a [Github](https://github.com/ironcalc) ticket or contact us via [email](mailto:hello@ironcalc.com) or [Discord](https://discord.com/invite/zZYWfh3RHJ).

## Error propagation

Some errors are created by some formulas. For instance, the function `SQRT` can create the error `#NUM!`, but can't ceate the error `#DIV/0`.

Once an error is created it is normally _propagated_ by all the formulas. So if cell `C3` evaluates to `#ERROR!`, then the formula
`=SQRT(C3)` will return `#ERROR!`.

Not all functions propagate errors in their arguments. For instance the function `IF(condition, if_true, if_false)` will only propagate an error in the `if_false` argument if the `condition` is `FALSE`. This is called _lazy evaluation_ - the function `IF` is _lazy_ because it only evaluates the arguments when needed. The opposite of lazy evaulaution is called _eager evaluation_.

Some functions also expect an error as an argument like [`ERROR.TYPE`](/functions/information/error.type) and will not propagate the error.


## See also

The following functions are convenient when working with errors

- [`ISERR(ref)`](/functions/information/iserr), `TRUE` if `ref` is any error type except the `#N/A` error.
- [`ISERROR(ref)`](/functions/information/iserror), `TRUE` if `ref` is any error.
- [`ISNA(ref)`](/functions/information/isna), `TRUE` if ref is `#N/A`.
- [`ERROR.TYPE`](/functions/information/error.type) returns the numeric code for a given error.
- [`IFERROR(ref, value)`](/functions/logical/iferror) returns `value` if the content of `ref` is an error.
- [`IFNA(ref, value)`](/functions/logical/ifna) returns `value` only if the content of `ref` is the `#N/A` error.
