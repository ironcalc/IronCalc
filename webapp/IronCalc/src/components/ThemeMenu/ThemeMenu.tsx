import type { IronCalcTheme } from "@ironcalc/wasm";
import { Settings } from "lucide-react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Menu } from "../Menu/Menu";
import { MenuDivider } from "../Menu/MenuDivider";
import { MenuItem } from "../Menu/MenuItem";
import { themeEquals } from "../RightDrawer/Themes/themeUtils";
import "./theme-menu.css";

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
  onManageThemes: () => void;
};

const ThemeMenu = ({
  children,
  themes,
  currentTheme,
  onChange,
  onManageThemes,
}: ThemeMenuProperties) => {
  const { t } = useTranslation();
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
              <span className="ic-theme-menu-swatches">
                {ACCENT_KEYS.map((key) => (
                  <span
                    key={key}
                    className="ic-theme-menu-swatch"
                    style={{ backgroundColor: theme[key] as string }}
                  />
                ))}
              </span>
            }
          >
            {theme.name}
          </MenuItem>
        );
      })}
      <MenuDivider />
      <MenuItem icon={<Settings />} onClick={onManageThemes}>
        {t("themes.manage_themes")}
      </MenuItem>
    </Menu>
  );
};

export default ThemeMenu;
