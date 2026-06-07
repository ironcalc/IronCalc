import type { IronCalcTheme } from "@ironcalc/wasm";
import type { ReactNode } from "react";
import { Menu } from "../Menu/Menu";
import { MenuItem } from "../Menu/MenuItem";

const ACCENT_KEYS: (keyof IronCalcTheme)[] = [
  "accent1",
  "accent2",
  "accent3",
  "accent4",
  "accent5",
  "accent6",
];

type ThemeMenuProperties = {
  children: ReactNode;
  themes: IronCalcTheme[];
  currentTheme: IronCalcTheme;
  onChange: (theme: IronCalcTheme) => void;
};

function themeEquals(theme1: IronCalcTheme, theme2: IronCalcTheme) {
  return (
    theme1.name === theme2.name &&
    ACCENT_KEYS.every((key) => theme1[key] === theme2[key]) &&
    theme1.dk1 === theme2.dk1 &&
    theme1.lt1 === theme2.lt1 &&
    theme1.dk2 === theme2.dk2 &&
    theme1.lt2 === theme2.lt2 &&
    theme1.fol_hlink === theme2.fol_hlink &&
    theme1.hlink === theme2.hlink
  );
}

const ThemeMenu = ({
  children,
  themes,
  currentTheme,
  onChange,
}: ThemeMenuProperties) => {
  let allThemes = themes;
  if (!themes.some((theme) => themeEquals(theme, currentTheme))) {
    allThemes = [...themes, currentTheme];
  }

  return (
    <Menu trigger={<div className="ic-format-menu-anchor">{children}</div>}>
      {allThemes.map((theme) => {
        const isCurrent = themeEquals(theme, currentTheme);
        const suffix = isCurrent ? " (current)" : "";
        const key = `${theme.name}${suffix}`;
        return (
          <MenuItem
            key={key}
            onClick={() => onChange(theme)}
            checked={isCurrent}
            secondaryText={
              <span style={{ display: "flex", gap: 2, alignItems: "center" }}>
                {ACCENT_KEYS.map((key) => (
                  <span
                    key={key}
                    style={{
                      display: "inline-block",
                      width: 10,
                      height: 10,
                      borderRadius: 2,
                      backgroundColor: theme[key] as string,
                    }}
                  />
                ))}
              </span>
            }
          >
            {theme.name}
          </MenuItem>
        );
      })}
    </Menu>
  );
};

export default ThemeMenu;
