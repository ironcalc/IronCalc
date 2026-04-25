---
layout: doc
outline: deep
lang: en-US
---

# INFO function

## Overview

INFO returns information about the current operating environment and the workbook.

::: tip Language note
The `type_text` argument is **always in English**, regardless of the workbook's locale or the user's display language. See [Regional Settings](/features/regional-settings.md) for more details.
:::

## Usage

### Syntax

**INFO(<span title="Text" style="color:#E53935">type_text</span>) => <span title="Text" style="color:#E53935">text</span>**

### Argument descriptions

- *type_text* ([text](/features/value-types#text), required). A string specifying which type of environment information to return. Always written in English. See the table of supported values below.

### Supported `type_text` values

| `type_text` | Returns |
|---|---|
| `"numfile"` | The number of worksheets in the current workbook. |
| `"recalc"` | The current recalculation mode. IronCalc always returns `"Automatic"`. |
| `"release"` | The version of IronCalc as a text string. |
| `"system"` | The name of the operating environment: `"browser"` in the web app, or the OS name (e.g. `"linux"`, `"windows"`, `"macos"`) in other contexts. |

The following `type_text` values are recognized but **not yet implemented** and return a [`#NIMPL!`](/features/error-types.md#nimpl) error: `"directory"`, `"origin"`, `"osversion"`.

::: info Case-insensitive
The `type_text` argument is case-insensitive. `"release"`, `"RELEASE"`, and `"Release"` all work the same way.
:::

### Returned value

INFO returns a [text](/features/value-types#text) string, or a number for `"numfile"`.

### Error conditions

- If the wrong number of arguments is supplied, INFO returns the [`#ERROR!`](/features/error-types.md#error) error.
- If `type_text` is not a recognized string, INFO returns the [`#VALUE!`](/features/error-types.md#value) error.
- If `type_text` is one of the unimplemented values (`"directory"`, `"origin"`, `"osversion"`), INFO returns the [`#NIMPL!`](/features/error-types.md#nimpl) error.

## Examples

| Formula | Result | Comment |
|---|---|---|
| `=INFO("release")` | `"0.7.1"` | IronCalc version (example) |
| `=INFO("system")` | `"browser"` | Running in the web app |
| `=INFO("numfile")` | `3` | Workbook has 3 sheets |
| `=INFO("recalc")` | `"Automatic"` | Always automatic in IronCalc |

## Links

- Visit Microsoft Excel's [INFO function](https://support.microsoft.com/en-us/office/info-function-725f259a-0e4b-49b3-8b52-58815c69acae) page.
- [Google Sheets](https://support.google.com/docs/answer/3093180) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/INFO) also provide versions of the INFO function.
