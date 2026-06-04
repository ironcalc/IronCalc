import { ArrowLeft, PencilLine, X } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { IconButton } from "../../Button/IconButton";
import { Tooltip } from "../../Tooltip/Tooltip";
import EditTheme, { type ThemeData } from "./EditTheme";
import ThemePreview from "./ThemePreview";
import "./themes.css";

const INITIAL_THEMES: ThemeData[] = [
  {
    name: "Default",
    textColor: "#272525",
    bgColor: "#FFFFFF",
    accentColors: [
      "#4472C4",
      "#ED7D31",
      "#A9D18E",
      "#FFC000",
      "#5B9BD5",
      "#70AD47",
    ],
  },
  {
    name: "Ocean",
    textColor: "#1A2B3C",
    bgColor: "#F0F7FF",
    accentColors: [
      "#0066CC",
      "#00A3BF",
      "#007A87",
      "#005F73",
      "#0096C7",
      "#48CAE4",
    ],
  },
  {
    name: "Forest",
    textColor: "#1B3A1F",
    bgColor: "#F0FFF4",
    accentColors: [
      "#2D6A4F",
      "#40916C",
      "#74C69D",
      "#95D5B2",
      "#52B788",
      "#1B4332",
    ],
  },
];

type ThemesProps = {
  onClose: () => void;
};

const Themes = ({ onClose }: ThemesProps) => {
  const { t } = useTranslation();
  const [themes, setThemes] = useState<ThemeData[]>(INITIAL_THEMES);
  const [selectedIndex, setSelectedIndex] = useState(0);
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
            onClick={() => setSelectedIndex(i)}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                setSelectedIndex(i);
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
