---
layout: doc
outline: deep
lang: en-US
---

# ENCODEURL function

::: warning
**Note:** This draft page is under construction 🚧
:::

## Overview
ENCODEURL is a function of the Text category that returns a [percent-encoded](https://en.wikipedia.org/wiki/Percent-encoding) version of a string, so that it can safely be used as part of a URL. For example, a space becomes `%20` and a slash becomes `%2F`.

## Usage

### Syntax
**ENCODEURL(<span title="String" style="color:#1E88E5">text</span>) => <span title="String" style="color:#1E88E5">encoded</span>**

### Argument descriptions
* *text* ([string](/features/value-types#strings), required). The text to encode. A [number](/features/value-types#numbers) or [boolean](/features/value-types#booleans) is first converted to its text representation (for example `1.5` becomes `"1.5"` and `TRUE` becomes `"TRUE"`). An empty cell is treated as the empty string.

### Additional guidance
- Uppercase and lowercase letters `A-Z` and `a-z`, the digits `0-9`, and the characters `-`, `.` and `_` are returned unchanged. Every other character is encoded.
- In line with Excel, and unlike [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986#section-2.3), the tilde `~` is encoded as `%7E`.
- Characters outside ASCII are encoded byte by byte using their UTF-8 representation. For example, `ü` becomes `%C3%BC` and `€` becomes `%E2%82%AC`.
- Hexadecimal digits in the result are uppercase.
- ENCODEURL encodes the whole string, including characters like `:`, `/`, `?`, `&` and `=` that have a special meaning in URLs. It is intended for encoding a single component of a URL, such as a path segment or a query parameter value, rather than a complete URL.
- If *text* is a [range](/features/value-types#ranges) or an [array](/features/value-types#arrays), ENCODEURL encodes each element and returns an array of the same shape.

### Returned value
ENCODEURL returns a [string](/features/value-types#strings) containing the percent-encoded text. When *text* is a range or an array, it returns an [array](/features/value-types#arrays) of strings.

### Error conditions
* In common with many other IronCalc functions, ENCODEURL propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then ENCODEURL returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the encoded text would be 32,767 characters or longer, then ENCODEURL returns the [`#VALUE!`](/features/error-types.md#value) error. Since every encoded character takes three characters in the result, this can happen with inputs much shorter than the maximum length of a cell's text.

<!--@include: ../markdown-snippets/error-type-details.txt-->

## Details
Numbers are converted to text in the same way as the `&` operator, using at most 15 significant digits. For example, `=ENCODEURL(1/3)` returns `0.333333333333333` and `=ENCODEURL(1E+20)` returns `1E%2B20`.

## Examples

| Formula | Result |
| --- | --- |
| `=ENCODEURL("a b")` | `a%20b` |
| `=ENCODEURL("http://example.com/Profit and Loss.xlsx")` | `http%3A%2F%2Fexample.com%2FProfit%20and%20Loss.xlsx` |
| `=ENCODEURL("AZaz09-._")` | `AZaz09-._` |
| `=ENCODEURL("~!*'()")` | `%7E%21%2A%27%28%29` |
| `=ENCODEURL("日本")` | `%E6%97%A5%E6%9C%AC` |
| `=ENCODEURL(1.5)` | `1.5` |
| `=ENCODEURL(TRUE)` | `TRUE` |
| `=ENCODEURL("")` | (empty string) |
| `=ENCODEURL(ENCODEURL("a b"))` | `a%2520b` |

## Links
* See also IronCalc's [CONCAT](/functions/text/concat.md) and [SUBSTITUTE](/functions/text/substitute.md) functions, which are often used together with ENCODEURL to build URLs.
* Visit Microsoft Excel's [ENCODEURL function](https://support.microsoft.com/en-us/excel/functions/encodeurl-function) page.
* Both [Google Sheets](https://support.google.com/docs/answer/9199778) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/ENCODEURL) provide versions of the ENCODEURL function.
