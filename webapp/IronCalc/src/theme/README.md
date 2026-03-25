# Themes in IronCalc

At the moment a theme in IronCalc is configured via css variables in `theme.css`.

Those values are *duplicated* in `theme.ts` and need to be in sync. That is because eventually want to remove the MUI dependency.
But to be able to do that need both the css variables AND the MUI theme.

As of writing to "theme" IronCalc you need to pass a `themeVariables` with the options you want to change. This API is bound to change in the near future though.

