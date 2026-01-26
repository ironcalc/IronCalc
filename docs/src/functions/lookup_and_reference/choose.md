---
layout: doc
outline: deep
lang: en-US
---

# CHOOSE function

## Overview

CHOOSE is a function of the **Logical** category that returns a value from a list of values based on a specified index number. It is useful when you want to select one option from multiple possible values without writing complex nested conditional logic.

Common use cases include:
- Mapping numeric codes to text labels
- Selecting different calculation results based on a position or index

## Usage

### Syntax

**CHOOSE(<span title="Number" style="color:#2F80ED">index_num</span>, <span title="Any" style="color:#EB5757">value1</span>, <span title="Any" style="color:#EB5757">value2</span>, …) => <span title="Any" style="color:#EB5757">result</span>**

### Argument descriptions

- _index_num_ ([number](/features/value-types#numbers), required). The position of the value to return. The first value has index `1`, the second value has index `2`, and so on.

- _value1_ ([any](/features/value-types), required). The value to return when _index_num_ is `1`.

- _value2_ ([any](/features/value-types), required). The value to return when _index_num_ is `2`.

- _valueN_ ([any](/features/value-types), required). Additional values to choose from. You can supply multiple values, each corresponding to a sequential index.

### Additional guidance

- Values can be of any type, including [numbers](/features/value-types#numbers), [text](/features/value-types#text), [booleans](/features/value-types#booleans), or even [arrays](/features/value-types#arrays).
- CHOOSE evaluates only the selected value, which can make it more efficient than deeply nested IF expressions.
- If your logic depends on conditions rather than fixed positions, consider using logical functions such as IF or IFS instead.
- CHOOSE evaluates its arguments **lazily**, meaning only the value corresponding to the selected _index_num_ is evaluated.

### Returned value

CHOOSE returns a value of the same type as the selected argument. The returned value corresponds to the position specified by the _index_num_ argument.

### Error conditions

- In common with many other IronCalc functions, CHOOSE propagates errors that are found in any of its arguments.
- If too few or too many arguments are supplied, CHOOSE returns the [`#ERROR!`](/features/error-types.md#error) error.
- If the value of _index_num_ is not (or cannot be converted to) a [number](/features/value-types#numbers), CHOOSE returns the [`#VALUE!`](/features/error-types.md#value) error.
- If _index_num_ is less than `1` or greater than the number of supplied values, CHOOSE returns the [`#VALUE!`](/features/error-types.md#value) error.

<!--@include: ../markdown-snippets/error-type-details.txt-->

## Details

- Conceptually, CHOOSE works like positional indexing into a list:
  - If `index_num = 1`, the result is `value1`
  - If `index_num = 2`, the result is `value2`
  - …
  - If `index_num = n`, the result is `valueN`

- Unlike lookup functions that search for matching keys, CHOOSE relies entirely on the numeric position provided by _index_num_.

- CHOOSE uses **lazy evaluation**: only the selected value is evaluated, and all other values are ignored. This allows CHOOSE to safely reference expressions that might otherwise produce errors (such as division by zero) as long as they are not selected.


## Examples

[See some examples in IronCalc](https://app.ironcalc.com/?example=choose).

## Links

- For more information about selection by index, see Wikipedia’s article on [array indexing](https://en.wikipedia.org/wiki/Array_data_structure#Indexing).
- See also IronCalc’s [IF](/functions/logical/if) and [IFS](/functions/logical/ifs) functions.
- Visit Microsoft Excel’s [CHOOSE function](https://support.microsoft.com/en-us/office/choose-function-fc5c184f-cb62-4ec7-a46e-38653b98f5bc) documentation.
- Both [Google Sheets](https://support.google.com/docs/answer/3093371) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/CHOOSE) provide equivalent CHOOSE functions.
