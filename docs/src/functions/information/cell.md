---
layout: doc
outline: deep
lang: en-US
---

# CELL function

## Overview

CELL returns information about the formatting, location, or contents of a cell. The type of information to return is specified by the `info_type` argument.

::: tip Language note
The `info_type` argument is **always in English**, regardless of the workbook's locale or the user's display language. See [Regional Settings](/features/regional-settings.md) for more details.
:::

## Usage

### Syntax

**CELL(<span title="Text" style="color:#E53935">info_type</span>, [<span title="Reference" style="color:#43A047">reference</span>]) => value**

### Argument descriptions

- *info_type* ([text](/features/value-types#text), required). A string specifying which type of cell information to return. Always written in English. See the table of supported values below.
- *reference* ([reference](/features/value-types#references), optional). The cell to get information about. If omitted, CELL uses the cell containing the formula.

### Supported `info_type` values

| `info_type` | Returns |
|---|---|
| `"address"` | The absolute reference of the first cell as text (e.g. `$A$1`). |
| `"col"` | The column number of the cell. |
| `"contents"` | The value of the upper-left cell in the reference. |
| `"row"` | The row number of the cell. |
| `"type"` | The type of data in the cell: `"b"` for blank, `"l"` for label (text), or `"v"` for value (number, boolean, or error). |

The following `info_type` values are recognized but **not yet implemented** and return a [`#VALUE!`](/features/error-types.md#value) error: `"color"`, `"filename"`, `"format"`, `"parentheses"`, `"prefix"`, `"protect"`, `"width"`.

::: info Case-insensitive
The `info_type` argument is case-insensitive. `"address"`, `"ADDRESS"`, and `"Address"` all work the same way.
:::

### Returned value

The return type depends on the `info_type` argument. It may be a number or text string.

### Error conditions

- If no argument or more than two arguments are supplied, CELL returns the [`#ERROR!`](/features/error-types.md#error) error.
- If `info_type` is not a recognized string, CELL returns the [`#VALUE!`](/features/error-types.md#value) error.
- If the `reference` argument is not a cell reference, CELL returns the [`#VALUE!`](/features/error-types.md#value) error.
- If `info_type` is `"address"` and `reference` is on a different sheet than the formula cell, CELL returns the [`#NIMPL!`](/features/error-types.md#nimpl) error (cross-sheet address not yet implemented).
- If `info_type` is one of the unimplemented values (`"color"`, `"filename"`, etc.), CELL returns the [`#VALUE!`](/features/error-types.md#value) error.

## Examples

| Formula | Result | Comment |
|---|---|---|
| `=CELL("row", B5)` | `5` | Row number of B5 |
| `=CELL("col", B5)` | `2` | Column number of B5 |
| `=CELL("address", B5)` | `$B$5` | Absolute address of B5 |
| `=CELL("type", A1)` | `"b"` | A1 is blank |
| `=CELL("type", A2)` | `"l"` | A2 contains text |
| `=CELL("type", A3)` | `"v"` | A3 contains a number |

## Links

- Visit Microsoft Excel's [CELL function](https://support.microsoft.com/en-us/office/cell-function-51bd39a5-f338-4dbe-a33f-955d67c2b2cf) page.
- [Google Sheets](https://support.google.com/docs/answer/3093266) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/CELL) also provide versions of the CELL function.
