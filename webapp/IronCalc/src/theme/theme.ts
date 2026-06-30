import type {
  IronCalcThemeVariables,
  PartialIronCalcThemeVariables,
} from "./themeVariables";

export const defaultThemeVariables: IronCalcThemeVariables = {
  "--typography-font-family": "Inter",
  "--typography-font-size": "12px",

  "--palette-common-black": "#272525",
  "--palette-common-white": "#FFF",

  "--palette-primary-main": "#F2994A",
  "--palette-primary-light": "#EFAA6D",
  "--palette-primary-dark": "#D68742",
  "--palette-primary-contrast-text": "#FFF",

  "--palette-secondary-main": "#2F80ED",
  "--palette-secondary-light": "#4E92EC",
  "--palette-secondary-dark": "#2B6EC8",
  "--palette-secondary-contrast-text": "#FFF",

  "--palette-error-main": "#EB5757",
  "--palette-error-light": "#E77A7A",
  "--palette-error-dark": "#CB4C4C",
  "--palette-error-contrast-text": "#FFF",

  "--palette-warning-main": "#F2C94C",
  "--palette-warning-light": "#EED384",
  "--palette-warning-dark": "#D6B244",
  "--palette-warning-contrast-text": "#FFF",

  "--palette-info-main": "#9E9E9E",
  "--palette-info-light": "#E0E0E0",
  "--palette-info-dark": "#757575",
  "--palette-info-contrast-text": "#FFF",

  "--palette-success-main": "#27AE60",
  "--palette-success-light": "#57BD82",
  "--palette-success-dark": "#239152",
  "--palette-success-contrast-text": "#FFF",

  "--palette-grey-50": "#F5F5F5",
  "--palette-grey-100": "#F2F2F2",
  "--palette-grey-200": "#EEEEEE",
  "--palette-grey-300": "#E0E0E0",
  "--palette-grey-400": "#BDBDBD",
  "--palette-grey-500": "#9E9E9E",
  "--palette-grey-600": "#757575",
  "--palette-grey-700": "#616161",
  "--palette-grey-800": "#424242",
  "--palette-grey-900": "#333333",
  "--palette-grey-a100": "#F2F2F2",
  "--palette-grey-a200": "#EEEEEE",
  "--palette-grey-a400": "#bdbdbd",
  "--palette-grey-a700": "#616161",

  "--palette-sheet-header-corner-background": "#FFF",
  "--palette-sheet-header-text-color": "#333",
  "--palette-sheet-header-background": "#FFF",
  "--palette-sheet-header-global-selector-color": "#EAECF4",
  "--palette-sheet-header-selected-background": "#EEEEEE",
  "--palette-sheet-header-full-selected-background": "#D3D6E9",
  "--palette-sheet-header-selected-color": "#333",
  "--palette-sheet-header-border-color": "#E0E0E0",
  "--palette-sheet-grid-color": "#E0E0E0",
  "--palette-sheet-grid-separator-color": "#E0E0E0",
  "--palette-sheet-default-text-color": "#2E414D",
  "--palette-sheet-outline-color": "#F2994A",
  "--palette-sheet-outline-editing-color": "#FBE0C9",
  "--palette-sheet-outline-background-color": "#F2994A1A",
  "--palette-sheet-default-cell-font-family":
    'Inter, "Adjusted Arial Fallback", sans-serif',
  "--palette-sheet-header-font":
    'bold 12px Inter, "Adjusted Arial Fallback", sans-serif',
};

export const darkThemeVariables: IronCalcThemeVariables = {
  "--typography-font-family": "Inter",
  "--typography-font-size": "12px",

  "--palette-common-black": "#E8E6E3",
  "--palette-common-white": "#1E1E1E",

  "--palette-primary-main": "#F2994A",
  "--palette-primary-light": "#F5AD6E",
  "--palette-primary-dark": "#D68742",
  "--palette-primary-contrast-text": "#1E1E1E",

  "--palette-secondary-main": "#4E92EC",
  "--palette-secondary-light": "#6EA6F0",
  "--palette-secondary-dark": "#2B6EC8",
  "--palette-secondary-contrast-text": "#FFF",

  "--palette-error-main": "#EF6B6B",
  "--palette-error-light": "#F08A8A",
  "--palette-error-dark": "#CB4C4C",
  "--palette-error-contrast-text": "#1E1E1E",

  "--palette-warning-main": "#F2C94C",
  "--palette-warning-light": "#F5D879",
  "--palette-warning-dark": "#D6B244",
  "--palette-warning-contrast-text": "#1E1E1E",

  "--palette-info-main": "#ABABAB",
  "--palette-info-light": "#C2C2C2",
  "--palette-info-dark": "#757575",
  "--palette-info-contrast-text": "#1E1E1E",

  "--palette-success-main": "#3DBE76",
  "--palette-success-light": "#5FCB8E",
  "--palette-success-dark": "#239152",
  "--palette-success-contrast-text": "#1E1E1E",

  "--palette-grey-50": "#2A2A2A",
  "--palette-grey-100": "#2F2F2F",
  "--palette-grey-200": "#363636",
  "--palette-grey-300": "#444444",
  "--palette-grey-400": "#5E5E5E",
  "--palette-grey-500": "#7A7A7A",
  "--palette-grey-600": "#9E9E9E",
  "--palette-grey-700": "#B5B5B5",
  "--palette-grey-800": "#CFCFCF",
  "--palette-grey-900": "#EDEDED",
  "--palette-grey-a100": "#2F2F2F",
  "--palette-grey-a200": "#363636",
  "--palette-grey-a400": "#5E5E5E",
  "--palette-grey-a700": "#B5B5B5",

  "--palette-sheet-header-corner-background": "#242424",
  "--palette-sheet-header-text-color": "#D0D0D0",
  "--palette-sheet-header-background": "#242424",
  "--palette-sheet-header-global-selector-color": "#2C2F3A",
  "--palette-sheet-header-selected-background": "#333333",
  "--palette-sheet-header-full-selected-background": "#39405A",
  "--palette-sheet-header-selected-color": "#F0F0F0",
  "--palette-sheet-header-border-color": "#3A3A3A",
  "--palette-sheet-grid-color": "#3A3A3A",
  "--palette-sheet-grid-separator-color": "#3A3A3A",
  "--palette-sheet-default-text-color": "#E4E4E4",
  "--palette-sheet-outline-color": "#F2994A",
  "--palette-sheet-outline-editing-color": "#4A331C",
  "--palette-sheet-outline-background-color": "#F2994A26",
  "--palette-sheet-default-cell-font-family":
    'Inter, "Adjusted Arial Fallback", sans-serif',
  "--palette-sheet-header-font":
    'bold 12px Inter, "Adjusted Arial Fallback", sans-serif',
};

export function setThemeVariables(
  variables: PartialIronCalcThemeVariables,
  target: HTMLElement = document.documentElement,
): void {
  for (const [name, value] of Object.entries(variables)) {
    if (value == null) {
      target.style.removeProperty(name);
    } else {
      target.style.setProperty(name, value);
    }
  }
}

export function unsetThemeVariables(
  target: HTMLElement = document.documentElement,
): void {
  for (const name of Object.keys(defaultThemeVariables)) {
    target.style.removeProperty(name);
  }
}
