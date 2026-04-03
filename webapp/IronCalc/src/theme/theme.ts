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
