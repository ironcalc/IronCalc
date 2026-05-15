# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: formula.spec.ts >> pressing enter goes to cell below
- Location: e2e/formula.spec.ts:45:1

# Error details

```
Error: expect(locator).toHaveScreenshot(expected) failed

Locator: locator('.ic-worksheet-sheet-canvas')
  1005 pixels (ratio 0.01 of all image pixels) are different.

  Snapshot: enter.png

Call log:
  - Expect "toHaveScreenshot(enter.png)" with timeout 5000ms
    - verifying given screenshot expectation
  - waiting for locator('.ic-worksheet-sheet-canvas')
    - locator resolved to <canvas width="1280" height="545" class="ic-worksheet-sheet-canvas"></canvas>
  - taking element screenshot
    - disabled all CSS animations
  - waiting for fonts to load...
  - fonts loaded
  - attempting scroll into view action
    - waiting for element to be stable
  - 1005 pixels (ratio 0.01 of all image pixels) are different.
  - waiting 100ms before taking screenshot
  - waiting for locator('.ic-worksheet-sheet-canvas')
    - locator resolved to <canvas width="1280" height="545" class="ic-worksheet-sheet-canvas"></canvas>
  - taking element screenshot
    - disabled all CSS animations
  - waiting for fonts to load...
  - fonts loaded
  - attempting scroll into view action
    - waiting for element to be stable
  - captured a stable screenshot
  - 1005 pixels (ratio 0.01 of all image pixels) are different.

```

# Page snapshot

```yaml
- generic [ref=e4]:
  - generic [ref=e5]:
    - button "Open sidebar" [ref=e6] [cursor=pointer]:
      - img [ref=e8]
    - generic [ref=e11]:
      - button "File" [ref=e12] [cursor=pointer]
      - button "Help" [ref=e14] [cursor=pointer]
    - textbox [ref=e17]: Workbook1
    - generic [ref=e18]:
      - button [ref=e19]:
        - img [ref=e20]
      - button "Share" [ref=e25] [cursor=pointer]:
        - img [ref=e27]
        - generic [ref=e33]: Share
  - generic [active] [ref=e35]:
    - generic [ref=e37]:
      - generic [ref=e38]:
        - button "Undo" [disabled] [ref=e39]:
          - img [ref=e41]
        - button "Redo" [disabled] [ref=e44]:
          - img [ref=e46]
      - generic [ref=e50]:
        - button "Copy styles" [ref=e51] [cursor=pointer]:
          - img [ref=e53]
        - button "Clear formatting" [ref=e57] [cursor=pointer]:
          - img [ref=e59]
      - generic [ref=e65]:
        - button "Format as currency" [ref=e66] [cursor=pointer]:
          - img [ref=e68]
        - button "Format as percentage" [ref=e70] [cursor=pointer]:
          - img [ref=e72]
        - button "Decrease decimal places" [ref=e76] [cursor=pointer]:
          - img [ref=e78]
        - button "Increase decimal places" [ref=e81] [cursor=pointer]:
          - img [ref=e83]
        - button "123" [ref=e89] [cursor=pointer]:
          - text: "123"
          - img [ref=e91]
      - generic [ref=e94]:
        - button "Decrease font size" [ref=e95] [cursor=pointer]:
          - img [ref=e97]
        - generic [ref=e98]: "13"
        - button "Increase font size" [ref=e99] [cursor=pointer]:
          - img [ref=e101]
      - generic [ref=e103]:
        - button "Bold" [ref=e104] [cursor=pointer]:
          - img [ref=e106]
        - button "Italic" [ref=e108] [cursor=pointer]:
          - img [ref=e110]
        - button "Underline" [ref=e112] [cursor=pointer]:
          - img [ref=e114]
        - button "Strikethrough" [ref=e116] [cursor=pointer]:
          - img [ref=e118]
      - generic [ref=e122]:
        - button "Font color" [ref=e123] [cursor=pointer]:
          - img [ref=e125]
        - button "Fill color" [ref=e128] [cursor=pointer]:
          - img [ref=e130]
        - button "Borders" [ref=e135] [cursor=pointer]:
          - img [ref=e137]
        - button "Conditional formatting" [ref=e139] [cursor=pointer]:
          - img [ref=e141]
      - generic [ref=e146]:
        - button "Align left" [ref=e147] [cursor=pointer]:
          - img [ref=e149]
        - button "Align center" [ref=e150] [cursor=pointer]:
          - img [ref=e152]
        - button "Align right" [ref=e153] [cursor=pointer]:
          - img [ref=e155]
        - button "Align top" [ref=e156] [cursor=pointer]:
          - img [ref=e158]
        - button "Align middle" [ref=e160] [cursor=pointer]:
          - img [ref=e162]
        - button "Align bottom" [pressed] [ref=e166] [cursor=pointer]:
          - img [ref=e168]
        - button "Wrap text" [ref=e170] [cursor=pointer]:
          - img [ref=e172]
      - generic [ref=e176]:
        - button "Show/hide grid lines" [ref=e177] [cursor=pointer]:
          - img [ref=e179]
        - button "toolbar.download_png" [ref=e182] [cursor=pointer]:
          - img [ref=e184]
    - generic [ref=e189]:
      - generic [ref=e190]:
        - button "B4" [ref=e194] [cursor=pointer]:
          - text: B4
          - img [ref=e196]
        - generic [ref=e199]:
          - img [ref=e201]
          - textbox [ref=e205]
      - generic [ref=e212]:
        - generic [ref=e213]: A
        - generic [ref=e215]: B
        - generic [ref=e217]: C
        - generic [ref=e219]: D
        - generic [ref=e221]: E
        - generic [ref=e223]: F
        - generic [ref=e225]: G
        - generic [ref=e227]: H
        - generic [ref=e229]: I
        - generic [ref=e231]: J
      - generic [ref=e253]:
        - generic [ref=e254]:
          - button "Add sheet" [ref=e255] [cursor=pointer]:
            - img [ref=e257]
          - button "Sheet list" [ref=e258] [cursor=pointer]:
            - img [ref=e260]
        - tab "Sheet1 Open sheet menu" [selected] [ref=e263] [cursor=pointer]:
          - generic [ref=e264]: Sheet1
          - button "Open sheet menu" [ref=e265]:
            - img [ref=e266]
        - button "en-US English" [ref=e269] [cursor=pointer]:
          - text: en-US
          - text: English
```

# Test source

```ts
  1  | import { expect, test } from "@playwright/test";
  2  | import {
  3  |   clickCell,
  4  |   getFormulaBarValue,
  5  |   waitForCanvas,
  6  | } from "./helpers";
  7  | 
  8  | test("entering =1+1 in A1 shows result 2", async ({ page }) => {
  9  |   await page.goto("/");
  10 |   await waitForCanvas(page);
  11 | 
  12 |   // Select A1 and type a formula.
  13 |   await clickCell(page, 1, 1);
  14 |   await page.keyboard.type("=1+1");
  15 |   await page.keyboard.press("Enter");
  16 | 
  17 |   // Re-select A1.
  18 |   await clickCell(page, 1, 1);
  19 | 
  20 |   // Formula bar shows the formula (standard spreadsheet behaviour).
  21 |   expect(await getFormulaBarValue(page)).toBe("=1+1");
  22 | 
  23 |   // The canvas renders the computed value: verify visually.
  24 |   await expect(page.locator(".ic-worksheet-sheet-canvas")).toHaveScreenshot("formula-result.png", {
  25 |     maxDiffPixels: 0
  26 |   });
  27 | });
  28 | 
  29 | test("entering text in B2 persists after navigating away and back", async ({
  30 |   page,
  31 | }) => {
  32 |   await page.goto("/");
  33 |   await waitForCanvas(page);
  34 | 
  35 |   await clickCell(page, 2, 2); // B2
  36 |   await page.keyboard.type("hello");
  37 |   await page.keyboard.press("Enter");
  38 | 
  39 |   // Move away then come back.
  40 |   await clickCell(page, 1, 1);
  41 |   await clickCell(page, 2, 2);
  42 |   expect(await getFormulaBarValue(page)).toBe("hello");
  43 | });
  44 | 
  45 | test("pressing enter goes to cell below", async ({ page }) => {
  46 |   await page.goto("/");
  47 |   await waitForCanvas(page);
  48 | 
  49 |   await clickCell(page, 2, 2); // B2
  50 |   await page.keyboard.press("Enter");
  51 |   await page.keyboard.press("Enter");
  52 | 
  53 |   // Visually check that C2 is selected
> 54 |   await expect(page.locator(".ic-worksheet-sheet-canvas")).toHaveScreenshot("enter.png", {
     |                                                            ^ Error: expect(locator).toHaveScreenshot(expected) failed
  55 |     maxDiffPixels: 0
  56 |   });
  57 | });
  58 | 
```