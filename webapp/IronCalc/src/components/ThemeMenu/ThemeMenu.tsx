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
  currentThemeName: string;
  onChange: (theme: IronCalcTheme) => void;
};

const ThemeMenu = ({
  children,
  themes,
  currentThemeName,
  onChange,
}: ThemeMenuProperties) => (
  <Menu trigger={<div className="ic-format-menu-anchor">{children}</div>}>
    {themes.map((theme) => (
      <MenuItem
        key={theme.name}
        checked={theme.name === currentThemeName}
        onClick={() => onChange(theme)}
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
    ))}
  </Menu>
);

export default ThemeMenu;
