---
layout: doc
outline: deep
lang: en-US
---

# COLUMN function
## Overview
The COLUMN Function in IronCalc is a lookup & reference formula that is used to query and return the column number of a referenced Column or Cell.
## Usage
### Syntax
**COLUMN(<span title="Reference" style="color:#1E88E5">reference</span>) => <span title="Number" style="color:#1E88E5">column</span>**
### Argument descriptions
* *reference* ([cell](/features/value-types#references), [optional](/features/optional-arguments.md)). The number of the cell you wish to reference the column number of.
### Additional guidance
* When referencing a range of cells, only the column number of the left most cell will be returned.
* You are also able to reference complete columns instead of individual cells.
### Returned value
COLUMN returns the [number](/features/value-types#numbers) of the specific cell or column which is being referenced.
### Error conditions
* IronCalc currently does not support the referencing of cells with names.
## Details
The COLUMN Function can only be used to display the correlating number of a single column within a Sheet. If you wish to show the number of columns used within a specific range, you can use the COLUMNS Function.
## Examples
### No Cell Reference
When no cell reference is made, the formula uses **=COLUMN()**. This will then output the column number of the cell where the formula is placed.<br><br>For example, if the formula is placed in cell A1, then "1" will be displayed.
### With Cell Reference
When a cell reference is made, the formula uses **=COLUMN([Referenced Cell])**. This will then output the column number of the referenced cell, regardless of where the formula is placed in the sheet.<br><br>For example, if the cell B1 is the referenced cell, "2" will be the output of the formula no matter where it is placed in the sheet.<br><br>**Note:** references do not always have to be specific cells, you can also reference complete columns. For example, **=COLUMN(B:B)** would also result in an output of "2".
### Range References
The COLUMN function can also be used to reference a range of Cells or Columns. In this case only the most left-hand column will be the resulting output.<br><br>For example, **=COLUMN(A1:J1)** will result in the ouput of "1".
## Links