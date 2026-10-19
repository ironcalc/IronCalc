# Themes in IronCalc

At the moment, a theme in IronCalc is configured via CSS variables defined in `theme.css`.
Those values are *duplicated* in `theme.ts` and must be kept in sync, because we eventually want to remove the MUI dependency.
However, to be able to do that, we currently need both the CSS variables and the MUI theme.
At the time of writing, to “theme” IronCalc you need to pass a `themeVariables` object with the options you want to change. This API is likely to change in the near future, though.

## Dark mode

Light mode is the source of truth: the workbook only ever stores light-mode colors.
Dark mode is two things:

* `darkThemeVariables`: the CSS variables for the chrome (menus, headers, grid lines, default text...).
* `colorScheme="dark"` on `IronCalc`: every *workbook* color (fills, fonts, borders, conditional formats,
  color scales, icon sets, sheet tab colors, theme accents, color picker swatches...) is shown through
  `invertColor` (`invertColor.ts`) instead of as is. Whatever the user picks on screen goes through
  `uninvertColor` before it is stored, so a file edited in dark mode looks the same in light mode.

`invertColor` works in OKLCH: it keeps the hue, flips the lightness so that pure white lands on the dark
background and pure black on the dark foreground, and keeps the chroma relative to the sRGB gamut so the
transform is invertible (up to 8-bit rounding). The knobs are in `DEFAULT_OPTIONS`; the Storybook story
"Theme/Color inversion" is a playground to tune them and `tests/invertColor.test.ts` pins the behaviour.

In code, components never call `invertColor` directly. They use the hooks in `colorScheme.ts`:

* `useResolveColor(theme)` resolves a workbook `Color` (hex, theme slot or none) to the hex to show.
* `useDisplayHex()` maps an already resolved hex (e.g. from `model.resolveColor`) to the hex to show.
* `fromDisplayHex(hex, scheme)` maps a hex the user typed or picked back to the one to store.

The canvas (`WorksheetCanvas`) receives `colorScheme` in its settings and does the same through
`displayColor`.
