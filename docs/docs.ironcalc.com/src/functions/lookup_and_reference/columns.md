---
layout: doc
outline: deep
lang: en-US
---

# COLUMNS function
## Overview
The COLUMNS function in IronCalc is a lookup & reference formula that is used to query and return the number of columns referenced in a particular range or array.
## Usage
### Syntax
**COLUMNS(<span title="Reference" style="color:#1E88E5">reference</span>) => <span title="Number" style="color:#1E88E5">columns</span>**
### Argument descriptions
* *reference* ([cell](/features/value-types#references)). The cells, columns, array, range, or [Named Range](/features/name-manager.html) which you wish to evaluate.
### Additional guidance
* When using COLUMNS a reference must be included.
* You are able to reference either complete columns or individual cells.
* When referencing [Named Range](/features/name-manager.html), the complete column must have the label. Referencing individual cells using Named Ranges in COLUMNS is not supported.
* When using a Named Range as a reference, the reference is not case sensitive.
* IronCalc supports the use of both *Absolute* ($A$1) and *Relative* (A1) references.
* Cross-sheet references are also supported.
* When referencing a range of columns or cells, if a cell or column within the range is deleted the count will automatically adjust. However, if the cell or column that is explicitly referenced is deleted an error will be thrown.
### Returned value
COLUMNS returns the [number](/features/value-types#numbers) of columns which are being referenced.
### Error conditions
* [`#ERROR!`](/features/error-types.html#error) is returned if no reference is included.
* [`#NAME?`](/features/error-types.html#name) is returned if a Named Range being referenced is deleted.
* [`#REF!`](/features/error-types.html#ref) is returned if a cell being referenced is deleted.
* [`#VALUE!`](/features/error-types.html#value) is returned if a column being referenced is deleted.
* [`#VALUE!`](/features/error-types.html#value) is returned if a cell name is being referenced.
* [`#VALUE!`](/features/error-types.html#value) is returned when referencing a Named Range in combination with an additional cell or column.
## Details
The COLUMNS function can only be used to display the correlating number of columns being referenced. If you wish to show the number of a single column within a Sheet, you can use the [COLUMN](/functions/lookup_and_reference/column) function.
## Examples
### Basic Range
When a range of cells is referenced, only the number of columns will display.<br><br>For example **=COLUMNS(A1:C1)** and **=COLUMNS(C1:E1)** will both output a value of "3".
### Named Ranges
When using COLUMNS, Named Ranges can only be referenced individually and not in combination with other cells or columns.<br><br>For example, **=COLUMNS(Range1)** will output the amount of columns contained within your Named Range. An error will be returned if you try to reference anything else within the paranthesis.
### Single Cell & Single Column References
When a single cell is referenced, such as **=COLUMNS(G1)**, an Output of "1" will always be the result. This result will also return when referencing single columns, for example **=COLUMNS(G:G)**.
## Links
* Visit Microsoft Excel's [Columns function](https://support.microsoft.com/en-us/office/columns-function-4e8e7b4e-e603-43e8-b177-956088fa48ca) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093374) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/COLUMNS) provide versions of the COLUMNS function.