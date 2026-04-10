# Themes in IronCalc

At the moment, a theme in IronCalc is configured via CSS variables defined in `theme.css`.
Those values are *duplicated* in `theme.ts` and must be kept in sync, because we eventually want to remove the MUI dependency.
However, to be able to do that, we currently need both the CSS variables and the MUI theme.
At the time of writing, to “theme” IronCalc you need to pass a `themeVariables` object with the options you want to change. This API is likely to change in the near future, though.
