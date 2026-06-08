import type { IronCalcTheme } from "@ironcalc/wasm";
import { ArrowLeft, PencilLine, X } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { IconButton } from "../../Button/IconButton";
import { Tooltip } from "../../Tooltip/Tooltip";
import EditTheme, { type ThemeData } from "./EditTheme";
import ThemePreview from "./ThemePreview";
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
  const [themes, setThemes] = useState<ThemeData[]>(() =>
    builtinThemes.map(themeToThemeData),
  );
  const [selectedIndex, setSelectedIndex] = useState(() => {
    const index = builtinThemes.findIndex(
      (theme) => theme.name === currentTheme.name,
    );
    return index === -1 ? 0 : index;
  });

  const selectTheme = (index: number) => {
    setSelectedIndex(index);
    onThemePicked(builtinThemes[index]);
  };
  const [editing, setEditing] = useState<{
    theme: ThemeData;
    index: number;
  } | null>(null);

  const handleSave = (data: ThemeData) => {
    if (editing === null) {
      return;
    }
    setThemes((prev) => {
      const next = [...prev];
      next[editing.index] = data;
      return next;
    });
    setEditing(null);
  };

  if (editing) {
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
            initialName={editing.theme.name}
            initialTextColor={editing.theme.textColor}
            initialBgColor={editing.theme.bgColor}
            initialLightColor={editing.theme.lightColor}
            initialDarkColor={editing.theme.darkColor}
            initialAccentColors={editing.theme.accentColors}
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
      <div className="ic-themes-content">
        {themes.map((theme, i) => (
          // biome-ignore lint/a11y/noStaticElementInteractions: FIXME
          <div
            // biome-ignore lint/suspicious/noArrayIndexKey: themes list has stable order
            key={i}
            className={`ic-themes-list-item${selectedIndex === i ? " ic-themes-list-item--selected" : ""}`}
            // biome-ignore lint/a11y/noNoninteractiveTabindex: FIXME
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
              theme={theme}
              className="ic-themes-list-item-preview"
            />
            <div className="ic-themes-list-item-name">{theme.name}</div>
            <div className="ic-themes-list-item-icons">
              <Tooltip title={t("themes.edit_theme")}>
                <IconButton
                  icon={<PencilLine />}
                  onClick={(e) => {
                    e.stopPropagation();
                    setEditing({ theme, index: i });
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
