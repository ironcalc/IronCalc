import { Check } from "lucide-react";
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import ColorPicker from "../../ColorPicker/ColorPicker";
import { resolveColorToHex } from "../../ColorPicker/util";
import { Input } from "../../Input/Input";
import ThemePreview from "./ThemePreview";
import "./edit-theme.css";
import type { IronCalcTheme } from "@ironcalc/wasm";

export interface ThemeData {
  name: string;
  textColor: string;
  bgColor: string;
  lightColor: string;
  darkColor: string;
  accentColors: [string, string, string, string, string, string];
  hlinkColor: string;
  folHlinkColor: string;
}

interface EditThemeProps {
  initialName: string;
  initialTextColor: string;
  initialBgColor: string;
  initialLightColor: string;
  initialDarkColor: string;
  currentTheme: IronCalcTheme;
  initialAccentColors: [string, string, string, string, string, string];
  initialHlinkColor: string;
  initialFolHlinkColor: string;
  onSave: (data: ThemeData) => void;
  onClose: () => void;
}

interface ColorFieldProps {
  label: string;
  value: string;
  onChange: (color: string) => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
  currentTheme: IronCalcTheme;
}

const ColorField = ({
  label,
  value,
  onChange,
  onKeyDown,
  currentTheme,
}: ColorFieldProps) => {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLButtonElement>(null);

  return (
    <div className="ic-edit-theme-field-wrapper">
      <span className="ic-edit-theme-label">{label}</span>
      <div className="ic-edit-theme-color-row">
        <div className="ic-input-control md ic-edit-theme-color-swatch-wrapper">
          <button
            ref={ref}
            type="button"
            className="ic-edit-theme-color-swatch"
            style={{ backgroundColor: value }}
            onClick={() => setOpen(true)}
            aria-label={label}
          />
        </div>
        <Input
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={onKeyDown}
        />
        <ColorPicker
          color={value}
          defaultColor={value}
          title={t("color_picker.default")}
          onChange={(c) => {
            onChange(resolveColorToHex(c, currentTheme));
            setOpen(false);
          }}
          onClose={() => setOpen(false)}
          anchorEl={ref}
          open={open}
          theme={currentTheme}
        />
      </div>
    </div>
  );
};

const EditTheme = ({
  initialName,
  initialTextColor,
  initialBgColor,
  initialLightColor,
  initialDarkColor,
  initialAccentColors,
  initialHlinkColor,
  initialFolHlinkColor,
  onSave,
  onClose,
  currentTheme,
}: EditThemeProps) => {
  const { t } = useTranslation();
  const [textColor, setTextColor] = useState(initialTextColor);
  const [bgColor, setBgColor] = useState(initialBgColor);
  const [lightColor, setLightColor] = useState(initialLightColor);
  const [darkColor, setDarkColor] = useState(initialDarkColor);
  const [accentColors, setAccentColors] = useState(initialAccentColors);
  const [hlinkColor, setHlinkColor] = useState(initialHlinkColor);
  const [folHlinkColor, setFolHlinkColor] = useState(initialFolHlinkColor);

  const setAccent = (index: number, color: string) => {
    setAccentColors((prev) => {
      const next = [...prev] as typeof prev;
      next[index] = color;
      return next;
    });
  };

  const handleSave = () => {
    onSave({
      name: initialName,
      textColor,
      bgColor,
      lightColor,
      darkColor,
      accentColors,
      hlinkColor,
      folHlinkColor,
    });
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    e.stopPropagation();
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      handleSave();
    }
  };

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: container captures Cmd/Ctrl+Enter shortcut bubbling from child inputs
    <div className="ic-edit-theme-container" onKeyDown={handleKeyDown}>
      <div className="ic-edit-theme-content-area">
        <div className="ic-edit-theme-header-box">
          <ThemePreview
            theme={{ bgColor, textColor, accentColors }}
            className="ic-edit-theme-preview"
          />
          <span className="ic-edit-theme-header-box-text">{initialName}</span>
        </div>
        <div className="ic-edit-theme-section">
          <ColorField
            label={t("themes.text_color_label")}
            value={textColor}
            onChange={setTextColor}
            onKeyDown={handleKeyDown}
            currentTheme={currentTheme}
          />
          <ColorField
            label={t("themes.bg_color_label")}
            value={bgColor}
            onChange={setBgColor}
            onKeyDown={handleKeyDown}
            currentTheme={currentTheme}
          />
        </div>
        <div className="ic-edit-theme-section">
          <ColorField
            label={t("themes.dark_color_label")}
            value={darkColor}
            onChange={setDarkColor}
            onKeyDown={handleKeyDown}
            currentTheme={currentTheme}
          />
          <ColorField
            label={t("themes.light_color_label")}
            value={lightColor}
            onChange={setLightColor}
            onKeyDown={handleKeyDown}
            currentTheme={currentTheme}
          />
        </div>
        <div className="ic-edit-theme-section">
          {accentColors.map((color, i) => (
            <ColorField
              // biome-ignore lint/suspicious/noArrayIndexKey: fixed-length array, order never changes
              key={i}
              label={t("themes.accent_color_label", { n: i + 1 })}
              value={color}
              onChange={(c) => setAccent(i, c)}
              onKeyDown={handleKeyDown}
              currentTheme={currentTheme}
            />
          ))}
        </div>
        <div className="ic-edit-theme-section">
          <ColorField
            label={t("themes.hlink_color_label")}
            value={hlinkColor}
            onChange={setHlinkColor}
            onKeyDown={handleKeyDown}
            currentTheme={currentTheme}
          />
          <ColorField
            label={t("themes.fol_hlink_color_label")}
            value={folHlinkColor}
            onChange={setFolHlinkColor}
            onKeyDown={handleKeyDown}
            currentTheme={currentTheme}
          />
        </div>
      </div>
      <div className="ic-edit-theme-footer">
        <Button variant="secondary" onClick={onClose}>
          {t("themes.cancel")}
        </Button>
        <Button startIcon={<Check />} onClick={handleSave}>
          {t("themes.apply")}
        </Button>
      </div>
    </div>
  );
};

export default EditTheme;
