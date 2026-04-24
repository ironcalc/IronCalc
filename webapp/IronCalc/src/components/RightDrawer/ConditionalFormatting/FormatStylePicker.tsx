import {
  Bold,
  Italic,
  PaintBucket,
  RemoveFormatting,
  Strikethrough,
  Type,
  Underline,
} from "lucide-react";
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { IconButton } from "../../Button/IconButton";
import ColorPicker from "../../ColorPicker/ColorPicker";
import "./format-style-picker.css";

export interface FormatStyle {
  bold: boolean;
  italic: boolean;
  underline: boolean;
  strike: boolean;
  fontColor: string;
  fillColor: string;
}

const DEFAULT_STYLE: FormatStyle = {
  bold: false,
  italic: false,
  underline: false,
  strike: false,
  fontColor: "#000000",
  fillColor: "",
};

const PRESETS: FormatStyle[] = [
  { ...DEFAULT_STYLE, fontColor: "#D21D21", fillColor: "#FBE7E8" },
  { ...DEFAULT_STYLE, fontColor: "#D21D21", fillColor: "" },
  { ...DEFAULT_STYLE, fontColor: "#8B6F00", fillColor: "#FEF9E8" },
  { ...DEFAULT_STYLE, fontColor: "#8B6F00", fillColor: "" },
  { ...DEFAULT_STYLE, fontColor: "#A5530A", fillColor: "#FCE7D4" },
  { ...DEFAULT_STYLE, fontColor: "#A5530A", fillColor: "" },
  { ...DEFAULT_STYLE, fontColor: "#008121", fillColor: "#F1F6EA" },
  { ...DEFAULT_STYLE, fontColor: "#008121", fillColor: "" },
];

interface FormatStylePickerProps {
  value: FormatStyle;
  onChange: (style: FormatStyle) => void;
}

const FormatStylePicker = ({ value, onChange }: FormatStylePickerProps) => {
  const { t } = useTranslation();
  const [fontColorOpen, setFontColorOpen] = useState(false);
  const [fillColorOpen, setFillColorOpen] = useState(false);
  const fontColorRef = useRef<HTMLButtonElement>(null);
  const fillColorRef = useRef<HTMLButtonElement>(null);

  const toggle = (
    key: keyof Pick<FormatStyle, "bold" | "italic" | "underline" | "strike">,
  ) => onChange({ ...value, [key]: !value[key] });

  const previewStyle: React.CSSProperties = {
    fontWeight: value.bold ? "bold" : "normal",
    fontStyle: value.italic ? "italic" : "normal",
    textDecoration:
      [value.underline ? "underline" : "", value.strike ? "line-through" : ""]
        .filter(Boolean)
        .join(" ") || "none",
    color: value.fontColor || "#000000",
    backgroundColor: value.fillColor || "transparent",
  };

  return (
    <div className="ic-fsp-wrapper">
      <div className="ic-fsp-editor">
        <div className="ic-fsp-preview" style={previewStyle}>
          AaBbCc
        </div>
        <div className="ic-fsp-toolbar">
          <div className="ic-fsp-toolbar-left">
            <IconButton
              icon={<Bold />}
              pressed={value.bold}
              aria-label={t("toolbar.bold")}
              onClick={() => toggle("bold")}
            />
            <IconButton
              icon={<Italic />}
              pressed={value.italic}
              aria-label={t("toolbar.italic")}
              onClick={() => toggle("italic")}
            />
            <IconButton
              icon={<Underline />}
              pressed={value.underline}
              aria-label={t("toolbar.underline")}
              onClick={() => toggle("underline")}
            />
            <IconButton
              icon={<Strikethrough />}
              pressed={value.strike}
              aria-label={t("toolbar.strike_through")}
              onClick={() => toggle("strike")}
            />
            <IconButton
              ref={fontColorRef}
              aria-label={t("toolbar.font_color")}
              onClick={() => setFontColorOpen(true)}
              icon={
                <>
                  <Type />
                  <div
                    className="ic-fsp-color-bar"
                    style={{ backgroundColor: value.fontColor || "#000000" }}
                  />
                </>
              }
            />
            <IconButton
              ref={fillColorRef}
              aria-label={t("toolbar.fill_color")}
              onClick={() => setFillColorOpen(true)}
              icon={
                <>
                  <PaintBucket />
                  <div
                    className="ic-fsp-color-bar"
                    style={{
                      backgroundColor: value.fillColor || "transparent",
                    }}
                  />
                </>
              }
            />
          </div>
          <IconButton
            icon={<RemoveFormatting />}
            aria-label={t("toolbar.clear_formatting")}
            onClick={() => onChange(DEFAULT_STYLE)}
          />
        </div>
      </div>

      <div className="ic-fsp-presets-section">
        <div className="ic-fsp-presets-label">
          {t("conditional_formatting.default_styles")}
        </div>
        <div className="ic-fsp-presets">
          {PRESETS.map((preset, index) => (
            <button
              // biome-ignore lint/suspicious/noArrayIndexKey: static preset list
              key={index}
              type="button"
              className="ic-fsp-preset"
              style={{
                color: preset.fontColor,
                backgroundColor: preset.fillColor || "#FFFFFF",
                borderColor: preset.fontColor,
              }}
              onClick={() => onChange(preset)}
              aria-label={t("conditional_formatting.apply_preset")}
            >
              Aa
            </button>
          ))}
        </div>
      </div>

      <ColorPicker
        color={value.fontColor}
        defaultColor="#000000"
        title={t("color_picker.default")}
        onChange={(color) => {
          onChange({ ...value, fontColor: color });
          setFontColorOpen(false);
        }}
        onClose={() => setFontColorOpen(false)}
        anchorEl={fontColorRef}
        open={fontColorOpen}
      />
      <ColorPicker
        color={value.fillColor || "#FFFFFF"}
        defaultColor=""
        title={t("color_picker.default")}
        onChange={(color) => {
          onChange({ ...value, fillColor: color });
          setFillColorOpen(false);
        }}
        onClose={() => setFillColorOpen(false)}
        anchorEl={fillColorRef}
        open={fillColorOpen}
      />
    </div>
  );
};

export default FormatStylePicker;
