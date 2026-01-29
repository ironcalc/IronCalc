---
layout: doc
outline: deep
lang: en-US
---

# Function Documentation Guide

This guide explains how to document IronCalc functions following our established format and style conventions.

## File Structure

Function documentation files should be placed in the appropriate category directory under `src/functions/`. For example:

- Financial functions: `src/functions/financial/function-name.md`
- Text functions: `src/functions/text/function-name.md`
- Logical functions: `src/functions/logical/function-name.md`

## Required Frontmatter

Every function documentation file must start with this frontmatter:

```yaml
---
layout: doc
outline: deep
lang: en-US
---
```

## Document Structure

A complete function documentation should include the following sections in order:

### 1. Title

The title should be the function name followed by the word "function":

```markdown
# FV function
```

The function name should be written in uppercase when mentioned in the documentation.

### 2. Draft Warning (Optional)

If the function hasn't been implemented, include this warning box:

```markdown
::: warning
**Note:** This draft page is under construction 🚧
:::
```

If the function has been implemented but not documented, include this warning box:

```markdown
::: warning
🚧 This function is implemented but currently lacks detailed documentation. For guidance, you may refer to the equivalent functionality in [Microsoft Excel documentation](https://support.microsoft.com/en-us/office/excel-functions-by-category-5f91f4e9-7b42-46d2-9bd1-63f26a86c0eb).
:::
```

### 3. Overview

Provide a brief, clear description of what the function does. If the function name is an acronym, expand it using underlined text:

```markdown
## Overview

FV (<u>F</u>uture <u>V</u>alue) is a function of the Financial category that can be used to predict the future value of an investment or asset based on its present value.
```

Include:

- Category (Financial, Text, Logical, etc.)
- Primary purpose
- Key use cases (if helpful)

### 4. Usage

This section contains multiple subsections:

#### 4.1 Syntax

Format the function syntax with color-coded argument types. Use the following color scheme:

- **Numbers**: `#2F80ED` (blue)
- **Booleans**: `#27AE60` (green)
- **Text/Strings**: `#2F80ED` (orange)
- **Arrays/Ranges**: `#EB5757` (red)

**Format:**

```markdown
### Syntax

**FUNCTION_NAME(<span title="Type" style="color:#HEXCODE">arg1</span>, <span title="Type" style="color:#HEXCODE">arg2</span>=default, ...) => <span title="ReturnType" style="color:#HEXCODE">return_value</span>**
```

**Example:**

**FV(<span title="Number" style="color:#2F80ED">rate</span>, <span title="Number" style="color:#2F80ED">nper</span>, <span title="Number" style="color:#2F80ED">pmt</span>, [<span title="Number" style="color:#2F80ED">pv</span>], [<span title="Boolean" style="color:#27AE60">type</span>]  => <span title="Number" style="color:#2F80ED">fv</span>**

```markdown
### Syntax

**FV(<span title="Number" style="color:#2F80ED">rate</span>, <span title="Number" style="color:#2F80ED">nper</span>, <span title="Number" style="color:#2F80ED">pmt</span>, <span title="Number" style="color:#2F80ED">pv</span>=0, <span title="Boolean" style="color:#27AE60">type</span>=FALSE) => <span title="Number" style="color:#2F80ED">fv</span>**
```

**Guidelines:**

- Use `title` attribute to specify the data type
- Use `style="color:#HEXCODE"` for syntax highlighting
- Use square brackets for optional arguments
- Show the return type after `=>`
- Make the entire syntax **bold**

#### 4.2 Argument Descriptions

List each argument with:

- Argument name in _italics_
- Data type link (e.g., `[number](/features/value-types#numbers)`)
- Required or optional indicator
- Description

**Format:**

```markdown
### Argument descriptions

- _argname_ ([datatype](/features/value-types#datatype), [required|optional](/features/optional-arguments.md)). Description of the argument.
```

**Example:**

```markdown
### Argument descriptions

- _rate_ ([number](/features/value-types#numbers), required). The fixed percentage interest rate or yield per period.
- _pv_ ([number](/features/value-types#numbers), [optional](/features/optional-arguments.md)). "pv" is the <u>p</u>resent <u>v</u>alue or starting amount of the asset (default 0).
- _type_ ([Boolean](/features/value-types#booleans), [optional](/features/optional-arguments.md)). A logical value indicating whether the payment due dates are at the end (FALSE or 0) of the compounding periods or at the beginning (TRUE or any non-zero value). The default is FALSE when omitted.
```

**Guidelines:**

- Use bullet points (`*`)
- Italicize argument names with `*argname*`
- Link to value types documentation
- Link to optional arguments page when applicable
- Expand acronyms in descriptions using `<u>` tags if helpful
- Mention default values for optional arguments

#### 4.3 Additional Guidance

Provide tips, best practices and important notes about using the function:

```markdown
### Additional guidance

- Make sure that the _rate_ argument specifies the interest rate or yield applicable to the compounding period.
- The _pmt_ and _pv_ arguments should be expressed in the same currency unit.
- To ensure a worthwhile result, one of the _pmt_ and _pv_ arguments should be non-zero.
```

#### 4.4 Returned Value

Describe what the function returns:

```markdown
### Returned value

FV returns a [number](/features/value-types#numbers) representing the future value expressed in the same [currency unit](/features/units) that was used for the _pmt_ and _pv_ arguments.
```

Include:

- Return type (with link to value types if applicable)
- Units or format if relevant
- Any important characteristics

#### 4.5 Error Conditions

List all error scenarios the function may encounter:

```markdown
### Error conditions

- In common with many other IronCalc functions, FV propagates errors that are found in any of its arguments.
- If too few or too many arguments are supplied, FV returns the [`#ERROR!`](/features/error-types.md#error) error.
- If the value of any of the _rate_, _nper_, _pmt_ or _pv_ arguments is not (or cannot be converted to) a [number](/features/value-types#numbers), then FV returns the [`#VALUE!`](/features/error-types.md#value) error.
- If the value of the _type_ argument is not (or cannot be converted to) a [Boolean](/features/value-types#booleans), then FV again returns the [`#VALUE!`](/features/error-types.md#value) error.
- For some combinations of valid argument values, FV may return a [`#NUM!`](/features/error-types.md#num) error or a [`#DIV/0!`](/features/error-types.md#div-0) error.
```

**Guidelines:**

- Use bullet points
- Format error types using backticks and link to the error types page: `` [`#ERROR!`](/features/error-types.md#error) ``
- Reference argument names in italics when discussing specific arguments
- Add the include directive at the end if using the error details snippet:

```markdown
<!--@include: ../markdown-snippets/error-type-details.txt-->
```

### 5. Details (Optional but Recommended)

For functions with mathematical formulas or complex behavior, include a Details section. This section can also include plots, graphs or charts to help clarify the function's behavior. 

```markdown
## Details

- If $\text{type} \neq 0$, $\text{fv}$ is given by the equation:
  $$ \text{fv} = -\text{pv} \times (1 + \text{rate})^\text{nper} - \dfrac{\text{pmt}\times\big({(1+\text{rate})^\text{nper}-1}\big) \times(1+\text{rate})}{\text{rate}}$$

- If $\text{type} = 0$, $\text{fv}$ is given by the equation:
  $$ \text{fv} = -\text{pv} \times (1 + \text{rate})^{\text{nper}} - \dfrac{\text{pmt}\times\big({(1+\text{rate})^\text{nper}-1}\big)}{\text{rate}}$$
```

**Guidelines:**

- Use LaTeX math notation with `$` for inline and `$$` for block equations
- Use `\text{}` for variable names in equations
- Explain special cases or edge conditions

### 6. Examples

Link to interactive examples in IronCalc:

```markdown
## Examples

[See some examples in IronCalc](https://app.ironcalc.com/?example=functionname).
```

Replace `functionname` with the actual function name (lowercase).

### 7. Links

Provide external references and related functions:

```markdown
## Links

- For more information about the concept of "future value" in finance, visit Wikipedia's [Future value](https://en.wikipedia.org/wiki/Future_value) page.
- See also IronCalc's [NPER](/functions/financial/nper), [PMT](/functions/financial/pmt), [PV](/functions/financial/pv) and [RATE](/functions/financial/rate) functions.
- Visit Microsoft Excel's [FV function](https://support.microsoft.com/en-gb/office/fv-function-2eef9f44-a084-4c61-bdd8-4fe4bb1b71b3) page.
- Both [Google Sheets](https://support.google.com/docs/answer/3093224) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/FV) provide versions of the FV function.
```

**Guidelines:**

- Include Wikipedia links for concepts when available
- Link to related IronCalc functions in the same category
- Include links to equivalent functions in Excel, Google Sheets, and LibreOffice Calc
- Use bullet points

## Syntax Coloring Reference

### Color Codes

| Data Type   | Hex Color | Usage                                   |
| ----------- | --------- | --------------------------------------- |
| Number      | `#2F80ED` | All numeric arguments and return values |
| Boolean     | `#27AE60` | TRUE/FALSE arguments                    |
| Text/String | `#F2994A` | Text arguments                          |
| Array/Range | `#EB5757` | Array or range arguments                |

### Syntax Highlighting Template

```html
<span title="Type" style="color:#HEXCODE">argument_name</span>
```

**Examples:**

- Number: `<span title="Number" style="color:#2F80ED">rate</span>`
- Boolean: `<span title="Boolean" style="color:#27AE60">type</span>`
- Text: `<span title="Text" style="color:#F2994A">text</span>`

## Formatting Conventions

### Text Formatting

- **Function names**: Use exact case as in IronCalc
- **Argument names**: Use _italics_ when referencing in prose
- **Acronyms**: Expand using `<u>` tags: `<u>F</u>uture <u>V</u>alue`
- **Code/values**: Use backticks for error codes: `` `#ERROR!` ``
- **Links**: Use descriptive link text, not raw URLs

### Section Headers

- Use `#` for the page title with the function name (e.g., FV Function)
- Use `##` for main sections (Overview, Usage, Details, Examples, Links)
- Use `###` for subsections (Syntax, Argument descriptions, etc.)

### Lists

- Use bullet points (`*`) for argument descriptions and error conditions
- Use numbered lists only when order matters

## Checklist

Before submitting a function documentation, ensure:

- [ ] Frontmatter is correct
- [ ] Title follows the format "FUNCTION_NAME function"
- [ ] Overview clearly explains the function's purpose
- [ ] Syntax is color-coded correctly
- [ ] All arguments are documented with correct types
- [ ] Required vs optional arguments are clearly marked
- [ ] Return value is described
- [ ] Error conditions are comprehensive
- [ ] Examples link is included
- [ ] Links section includes relevant references
- [ ] Mathematical formulas (if any) use proper LaTeX syntax
- [ ] All internal links use relative paths
- [ ] Spelling and grammar are correct

## Example Template

```markdown
---
layout: doc
outline: deep
lang: en-US
---

# FUNCTION_NAME function

::: warning
**Note:** This draft page is under construction 🚧
:::

## Overview

FUNCTION_NAME (<u>A</u>cronym <u>E</u>xplanation) is a function of the [Category] category that can be used to [primary purpose].

[Additional context about when to use this function or related functions.]

## Usage

### Syntax

**FUNCTION_NAME(<span title="Type" style="color:#2F80ED">arg1</span>, [<span title="Type" style="color:#2F80ED">arg2</span>], [<span title="Boolean" style="color:#27AE60">arg3</span>]) => <span title="Type" style="color:#2F80ED">return_value</span>**

### Argument descriptions

- _arg1_ ([type](/features/value-types#type), required). Description.
- _arg2_ ([type](/features/value-types#type), [optional](/features/optional-arguments.md)). Description (default value).
- _arg3_ ([Boolean](/features/value-types#booleans), [optional](/features/optional-arguments.md)). Description (default FALSE).

### Additional guidance

- Tip or best practice.
- Another important note.

### Returned value

FUNCTION_NAME returns a [type](/features/value-types#type) representing [description].

### Error conditions

- General error propagation note.
- Specific error condition with [`#ERROR!`](/features/error-types.md#error) link.
- Another error condition.

<!--@include: ../markdown-snippets/error-type-details.txt-->

## Details

[Mathematical formulas or detailed explanations if needed]

## Examples

[See some examples in IronCalc](https://app.ironcalc.com/?example=functionname).

## Links

- Wikipedia link if applicable.
- Related IronCalc functions.
- Microsoft Excel documentation.
- Google Sheets and LibreOffice Calc links.
```

## Questions?

If you have questions about documenting functions, reach out on our [Discord Channel](https://discord.com/invite/zZYWfh3RHJ) or check existing function documentation for examples.
