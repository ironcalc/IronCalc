import type { IronCalcTheme } from "@ironcalc/wasm";
import { ArrowLeft, PencilLine, X } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { IconButton } from "../../Button/IconButton";
import { Tooltip } from "../../Tooltip/Tooltip";
import EditTheme, { type ThemeData } from "./EditTheme";
import ThemePreview from "./ThemePreview";
import { themeEquals } from "./themeUtils";
import "./themes.css";

function themeToThemeData(theme: IronCalcTheme): ThemeData {
  return {
    name: theme.name,
    textColor: theme.dk1,
    bgColor: theme.lt1,
    darkColor: theme.dk2,
    lightColor: theme.lt2,
    accentColors: [
      theme.accent1,
      theme.accent2,
      theme.accent3,
      theme.accent4,
      theme.accent5,
      theme.accent6,
    ],
  };
}

type ThemesProps = {
  themes: IronCalcTheme[];
  currentTheme: IronCalcTheme;
  onThemePicked: (theme: IronCalcTheme) => void;
  onClose: () => void;
};

const Themes = ({
  themes: builtinThemes,
  currentTheme,
  onThemePicked,
  onClose,
}: ThemesProps) => {
  const { t } = useTranslation();
  const [themes, setThemes] = useState<IronCalcTheme[]>(builtinThemes);
  const [selectedIndex, setSelectedIndex] = useState(() => {
    const index = themes.findIndex((theme) => themeEquals(theme, currentTheme));
    return index === -1 ? 0 : index;
  });

  const selectTheme = (index: number) => {
    setSelectedIndex(index);
    onThemePicked(themes[index]);
  };
  const [editing, setEditing] = useState<IronCalcTheme | null>(null);

  const handleSave = (data: ThemeData) => {
    if (editing === null) {
      return;
    }
    const customThemeName = t("themes.custom_theme_name");
    const newTheme: IronCalcTheme = {
      ...editing,
      name: customThemeName,
      dk1: data.textColor,
      lt1: data.bgColor,
      dk2: data.darkColor,
      lt2: data.lightColor,
      accent1: data.accentColors[0],
      accent2: data.accentColors[1],
      accent3: data.accentColors[2],
      accent4: data.accentColors[3],
      accent5: data.accentColors[4],
      accent6: data.accentColors[5],
    };

    // Only one custom theme at a time. Always placed on top.
    const next = [
      newTheme,
      ...themes.filter((theme) => theme.name !== customThemeName),
    ];
    setThemes(next);
    setSelectedIndex(0);
    onThemePicked(newTheme);
    setEditing(null);
  };

  if (editing) {
    const editingData = themeToThemeData(editing);
    return (
      <div className="ic-themes-container">
        <div className="ic-themes-edit-header">
          <Tooltip title={t("themes.back_to_list")}>
            <IconButton
              icon={<ArrowLeft />}
              onClick={() => setEditing(null)}
              aria-label={t("themes.back_to_list")}
            />
          </Tooltip>
          <div className="ic-themes-edit-header-title">
            {t("themes.edit_theme")}
          </div>
          <Tooltip title={t("right_drawer.close")}>
            <IconButton
              icon={<X />}
              onClick={onClose}
              aria-label={t("right_drawer.close")}
            />
          </Tooltip>
        </div>
        <div className="ic-themes-content">
          <EditTheme
            initialName={editingData.name}
            initialTextColor={editingData.textColor}
            initialBgColor={editingData.bgColor}
            initialLightColor={editingData.lightColor}
            initialDarkColor={editingData.darkColor}
            initialAccentColors={editingData.accentColors}
            currentTheme={currentTheme}
            onSave={handleSave}
            onClose={() => setEditing(null)}
          />
        </div>
      </div>
    );
  }

  return (
    <div className="ic-themes-container">
      <div className="ic-themes-header">
        <div className="ic-themes-header-title">{t("themes.panel_title")}</div>
        <Tooltip title={t("right_drawer.close")}>
          <IconButton
            icon={<X />}
            onClick={onClose}
            aria-label={t("right_drawer.close")}
          />
        </Tooltip>
      </div>
      <div
        className="ic-themes-content"
        role="listbox"
        aria-label={t("themes.panel_title")}
      >
        {themes.map((theme, i) => (
          <div
            // biome-ignore lint/suspicious/noArrayIndexKey: themes list has stable order
            key={i}
            className={`ic-themes-list-item${selectedIndex === i ? " ic-themes-list-item--selected" : ""}`}
            role="option"
            aria-selected={selectedIndex === i}
            aria-label={theme.name}
            tabIndex={0}
            onClick={() => selectTheme(i)}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                selectTheme(i);
              }
            }}
          >
            <ThemePreview
              theme={themeToThemeData(theme)}
              className="ic-themes-list-item-preview"
            />
            <div className="ic-themes-list-item-name">{theme.name}</div>
            <div className="ic-themes-list-item-icons">
              <Tooltip title={t("themes.edit_theme")}>
                <IconButton
                  icon={<PencilLine />}
                  onClick={(e) => {
                    e.stopPropagation();
                    setEditing(theme);
                  }}
                  aria-label={t("themes.edit_theme")}
                />
              </Tooltip>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default Themes;
