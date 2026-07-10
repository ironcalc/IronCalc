import type {
  CellStyle,
  FmtSettings,
  HorizontalAlignment,
  IronCalcTheme,
  Model,
  StyleIncludes,
  VerticalAlignment,
} from "@ironcalc/wasm";
import {
  AlignCenter,
  AlignLeft,
  AlignRight,
  ArrowDownToLine,
  ArrowUpToLine,
  Bold,
  Check,
  Italic,
  Strikethrough,
  Underline,
} from "lucide-react";
import { type CSSProperties, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { ArrowMiddleFromLine } from "../../../icons";
import { Button } from "../../Button/Button";
import { IconButton } from "../../Button/IconButton";
import ColorPicker from "../../ColorPicker/ColorPicker";
import { resolveColorToHex } from "../../ColorPicker/util";
import { NumberFormats } from "../../FormatMenu/formatUtil";
import { Input } from "../../Input/Input";
import { Select } from "../../Select/Select";
import { Toggle } from "../../Toggle/Toggle";
import type { FormatStyle } from "../ConditionalFormatting/FormatStylePicker";
import "./edit-named-style.css";
import { getPreviewText } from "./named-styles-utils";

export interface SaveError {
  nameError: string;
}

interface EditNamedStyleProps {
  model: Model;
  name: string;
  style: CellStyle;
  formatOptions: FmtSettings;
  existingStyleNames: string[];
  currentTheme: IronCalcTheme;
  onSave: (payload: NamedStyleSavePayload) => SaveError;
  onClose: () => void;
}

export interface NamedStyleSavePayload {
  name: string;
  style: CellStyle;
  includes: StyleIncludes;
}

function formatStyleToPreview(
  formatStyle: FormatStyle,
  currentTheme: IronCalcTheme,
  includeFont: boolean,
  includeFill: boolean,
): CSSProperties {
  const preview: CSSProperties = {};
  if (includeFill) {
    preview.backgroundColor =
      resolveColorToHex(formatStyle.fillColor, currentTheme) || undefined;
  }
  if (includeFont) {
    const decorations: string[] = [];
    if (formatStyle.underline) {
      decorations.push("underline");
    }
    if (formatStyle.strike) {
      decorations.push("line-through");
    }
    preview.color =
      resolveColorToHex(formatStyle.fontColor, currentTheme) || undefined;
    preview.fontWeight = formatStyle.bold ? "bold" : undefined;
    preview.fontStyle = formatStyle.italic ? "italic" : undefined;
    preview.textDecoration =
      decorations.length > 0 ? decorations.join(" ") : undefined;
  }
  return preview;
}

function initFormatStyle(model: Model, style: CellStyle): FormatStyle {
  return {
    bold: style.font.b,
    italic: style.font.i,
    underline: style.font.u,
    strike: style.font.strike,
    fontColor: model.resolveColor(style.font.color),
    fillColor: model.resolveColor(style.fill.color),
  };
}

const CUSTOM_VALUE = "__custom__";

const EditNamedStyle = ({
  model,
  name: initialName,
  style,
  formatOptions,
  existingStyleNames,
  currentTheme,
  onSave,
  onClose,
}: EditNamedStyleProps) => {
  const { t } = useTranslation();

  const getDefaultName = () => {
    if (initialName) {
      return initialName;
    }
    const prefix = t("named_styles.default_name_prefix");
    const existing = new Set(existingStyleNames.map((n) => n.toLowerCase()));
    let counter = 1;
    let candidate = `${prefix}${counter}`;
    while (existing.has(candidate.toLowerCase())) {
      counter++;
      candidate = `${prefix}${counter}`;
    }
    return candidate;
  };

  const knownFormats = [
    NumberFormats.AUTO,
    formatOptions.number_fmt,
    NumberFormats.PERCENTAGE,
    NumberFormats.CURRENCY_EUR,
    NumberFormats.CURRENCY_USD,
    NumberFormats.CURRENCY_GBP,
    formatOptions.short_date,
    formatOptions.long_date,
  ];

  const [name, setName] = useState(getDefaultName);
  const [nameError, setNameError] = useState("");
  const [formatStyle, setFormatStyle] = useState<FormatStyle>(() =>
    initFormatStyle(model, style),
  );
  const [numFmt, setNumFmt] = useState<string>(style.num_fmt);
  const [customFmt, setCustomFmt] = useState(() =>
    knownFormats.includes(style.num_fmt) ? "" : style.num_fmt,
  );
  const [customFmtTouched, setCustomFmtTouched] = useState(false);
  const [initialIncludes] = useState<StyleIncludes | null>(() => {
    if (!initialName) {
      return null;
    }
    try {
      return model.getNamedStyleIncludes(initialName);
    } catch {
      return null;
    }
  });
  const [includeFormat, setIncludeFormat] = useState(
    initialIncludes?.number_format ?? true,
  );
  const [includeFont, setIncludeFont] = useState(initialIncludes?.font ?? true);
  const [includeFill, setIncludeFill] = useState(initialIncludes?.fill ?? true);
  const [includeAlignment, setIncludeAlignment] = useState(
    initialIncludes?.alignment ?? true,
  );
  const [horizontalAlign, setHorizontalAlign] = useState<HorizontalAlignment>(
    style.alignment?.horizontal ?? "general",
  );
  const [verticalAlign, setVerticalAlign] = useState<VerticalAlignment>(
    style.alignment?.vertical ?? "bottom",
  );
  const [includeBorder, setIncludeBorder] = useState(
    initialIncludes?.border ?? true,
  );
  const [fontColorOpen, setFontColorOpen] = useState(false);
  const [fillColorOpen, setFillColorOpen] = useState(false);
  const fontColorRef = useRef<HTMLButtonElement>(null);
  const fillColorRef = useRef<HTMLButtonElement>(null);
  const customFmtInputRef = useRef<HTMLInputElement>(null);

  const toggleFontAttr = (
    key: keyof Pick<FormatStyle, "bold" | "italic" | "underline" | "strike">,
  ) => setFormatStyle((current) => ({ ...current, [key]: !current[key] }));

  const isCustom = !knownFormats.includes(numFmt);
  const wasCustomRef = useRef(isCustom);

  useEffect(() => {
    if (isCustom && !wasCustomRef.current) {
      customFmtInputRef.current?.focus();
    }
    wasCustomRef.current = isCustom;
  }, [isCustom]);

  const formatSelectOptions = [
    { value: NumberFormats.AUTO, label: t("toolbar.format_menu.auto") },
    { value: formatOptions.number_fmt, label: t("toolbar.format_menu.number") },
    {
      value: NumberFormats.PERCENTAGE,
      label: t("toolbar.format_menu.percentage"),
    },
    {
      value: NumberFormats.CURRENCY_EUR,
      label: t("toolbar.format_menu.currency_eur"),
    },
    {
      value: NumberFormats.CURRENCY_USD,
      label: t("toolbar.format_menu.currency_usd"),
    },
    {
      value: NumberFormats.CURRENCY_GBP,
      label: t("toolbar.format_menu.currency_gbp"),
    },
    {
      value: formatOptions.short_date,
      label: t("toolbar.format_menu.date_short"),
    },
    {
      value: formatOptions.long_date,
      label: t("toolbar.format_menu.date_long"),
    },
    { value: CUSTOM_VALUE, label: t("toolbar.format_menu.custom") },
  ];

  const handleFormatChange = (value: string) => {
    if (value === CUSTOM_VALUE) {
      setCustomFmtTouched(false);
      setNumFmt(customFmt || "");
    } else {
      setNumFmt(value);
    }
  };

  const selectValue = isCustom ? CUSTOM_VALUE : numFmt;
  const customFmtError = includeFormat && isCustom && !customFmt.trim();
  const hasError = !!nameError || !name.trim() || customFmtError;

  const handleSave = () => {
    if (hasError) {
      setCustomFmtTouched(true);
      return;
    }
    const newStyle = {
      ...style,
      num_fmt: numFmt,
      alignment: {
        horizontal: horizontalAlign,
        vertical: verticalAlign,
        wrap_text: style.alignment?.wrap_text ?? false,
      },
      fill: {
        ...style.fill,
        color: formatStyle.fillColor || undefined,
      },
      font: {
        ...style.font,
        b: formatStyle.bold || false,
        i: formatStyle.italic || false,
        u: formatStyle.underline || false,
        strike: formatStyle.strike || false,
        color: formatStyle.fontColor,
      },
    };
    // Protection is not editable in this panel, so it is always carried
    // over from the base style.
    const error = onSave({
      name: name.trim(),
      style: newStyle,
      includes: {
        number_format: includeFormat,
        font: includeFont,
        fill: includeFill,
        border: includeBorder,
        alignment: includeAlignment,
        protection: true,
      },
    });
    if (error.nameError) {
      setNameError(error.nameError);
    } else {
      onClose();
    }
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
    <div className="ic-edit-style-container" onKeyDown={handleKeyDown}>
      <div className="ic-edit-style-content-area">
        <div className="ic-edit-style-header-box">
          <div
            className="ic-edit-style-preview"
            style={formatStyleToPreview(
              formatStyle,
              currentTheme,
              includeFont,
              includeFill,
            )}
          >
            {getPreviewText(
              includeFormat ? numFmt : NumberFormats.AUTO,
              formatOptions,
              t,
            )}
          </div>
          <span className="ic-edit-style-header-box-text">
            {name.trim() || t("named_styles.new_style")}
          </span>
        </div>
        <div className="ic-edit-style-styled-box">
          <Input
            autoFocus
            type="text"
            label={t("named_styles.name_label")}
            placeholder={t("named_styles.name_placeholder")}
            value={name}
            error={!!nameError}
            helperText={nameError}
            onChange={(e) => {
              setName(e.target.value);
              setNameError("");
            }}
            onKeyDown={handleKeyDown}
          />
        </div>

        <div className="ic-edit-style-styled-box ic-edit-style-section-header">
          <div className="ic-edit-style-section-title">
            {t("named_styles.style_properties")}
          </div>
          <div className="ic-edit-style-section-subtitle">
            {t("named_styles.style_properties_description")}
          </div>
        </div>

        <div className="ic-edit-style-styled-box ic-edit-style-format-group">
          <Toggle
            checked={includeFormat}
            onChange={setIncludeFormat}
            label={t("named_styles.number_label")}
          />
          {includeFormat && (
            <>
              <Select
                label={t("named_styles.format_label")}
                value={selectValue}
                options={formatSelectOptions}
                onChange={handleFormatChange}
              />
              {isCustom && (
                <Input
                  ref={customFmtInputRef}
                  type="text"
                  placeholder={t("named_styles.custom_format_placeholder")}
                  value={customFmt}
                  error={customFmtTouched && customFmtError}
                  helperText={
                    customFmtTouched && customFmtError
                      ? t("named_styles.custom_format_required")
                      : ""
                  }
                  onChange={(e) => {
                    setCustomFmt(e.target.value);
                    setNumFmt(e.target.value);
                  }}
                  onBlur={() => setCustomFmtTouched(true)}
                  onKeyDown={handleKeyDown}
                />
              )}
            </>
          )}
        </div>
        <div className="ic-edit-style-styled-box ic-edit-style-format-group">
          <Toggle
            checked={includeFont}
            onChange={(checked) => {
              setIncludeFont(checked);
              if (!checked) {
                setFontColorOpen(false);
              }
            }}
            label={t("named_styles.font_label")}
          />
          {includeFont && (
            <>
              <div className="ic-edit-style-subrow">
                <span className="ic-edit-style-sublabel">
                  {t("named_styles.font_style_label")}
                </span>
                <div className="ic-edit-style-button-group">
                  <IconButton
                    icon={<Bold />}
                    pressed={formatStyle.bold}
                    aria-label={t("toolbar.bold")}
                    onClick={() => toggleFontAttr("bold")}
                  />
                  <IconButton
                    icon={<Italic />}
                    pressed={formatStyle.italic}
                    aria-label={t("toolbar.italic")}
                    onClick={() => toggleFontAttr("italic")}
                  />
                  <IconButton
                    icon={<Underline />}
                    pressed={formatStyle.underline}
                    aria-label={t("toolbar.underline")}
                    onClick={() => toggleFontAttr("underline")}
                  />
                  <IconButton
                    icon={<Strikethrough />}
                    pressed={formatStyle.strike}
                    aria-label={t("toolbar.strike_through")}
                    onClick={() => toggleFontAttr("strike")}
                  />
                </div>
              </div>
              <div className="ic-edit-style-subrow">
                <span className="ic-edit-style-sublabel">
                  {t("named_styles.font_color_label")}
                </span>
                <div className="ic-input-control md ic-edit-style-swatch-wrapper">
                  <button
                    ref={fontColorRef}
                    type="button"
                    className="ic-edit-style-swatch"
                    style={{
                      backgroundColor:
                        resolveColorToHex(
                          formatStyle.fontColor,
                          currentTheme,
                        ) || "#000000",
                    }}
                    onClick={() => setFontColorOpen(true)}
                    aria-label={t("toolbar.font_color")}
                  />
                </div>
                <ColorPicker
                  color={formatStyle.fontColor}
                  defaultColor="#000000"
                  title={t("color_picker.default")}
                  onChange={(color) => {
                    setFormatStyle((current) => ({
                      ...current,
                      fontColor: color,
                    }));
                    setFontColorOpen(false);
                  }}
                  onClose={() => setFontColorOpen(false)}
                  anchorEl={fontColorRef}
                  open={fontColorOpen}
                  theme={currentTheme}
                />
              </div>
            </>
          )}
        </div>
        <div className="ic-edit-style-styled-box ic-edit-style-format-group">
          <Toggle
            checked={includeFill}
            onChange={(checked) => {
              setIncludeFill(checked);
              if (!checked) {
                setFillColorOpen(false);
              }
            }}
            label={t("named_styles.fill_label")}
          />
          {includeFill && (
            <div className="ic-edit-style-subrow">
              <span className="ic-edit-style-sublabel">
                {t("named_styles.background_color_label")}
              </span>
              <div className="ic-input-control md ic-edit-style-swatch-wrapper">
                <button
                  ref={fillColorRef}
                  type="button"
                  className="ic-edit-style-swatch"
                  style={{
                    backgroundColor:
                      resolveColorToHex(formatStyle.fillColor, currentTheme) ||
                      "transparent",
                  }}
                  onClick={() => setFillColorOpen(true)}
                  aria-label={t("toolbar.fill_color")}
                />
              </div>
              <ColorPicker
                color={formatStyle.fillColor || "#FFFFFF"}
                defaultColor=""
                title={t("color_picker.default")}
                onChange={(color) => {
                  setFormatStyle((current) => ({
                    ...current,
                    fillColor: color,
                  }));
                  setFillColorOpen(false);
                }}
                onClose={() => setFillColorOpen(false)}
                anchorEl={fillColorRef}
                open={fillColorOpen}
                theme={currentTheme}
              />
            </div>
          )}
        </div>
        <div className="ic-edit-style-styled-box ic-edit-style-format-group">
          <Toggle
            checked={includeAlignment}
            onChange={setIncludeAlignment}
            label={t("named_styles.alignment_label")}
          />
          {includeAlignment && (
            <>
              <div className="ic-edit-style-subrow">
                <span className="ic-edit-style-sublabel">
                  {t("named_styles.horizontal_align_label")}
                </span>
                <div className="ic-edit-style-button-group">
                  <IconButton
                    icon={<AlignLeft />}
                    pressed={horizontalAlign === "left"}
                    aria-label={t("toolbar.align_left")}
                    onClick={() =>
                      setHorizontalAlign(
                        horizontalAlign === "left" ? "general" : "left",
                      )
                    }
                  />
                  <IconButton
                    icon={<AlignCenter />}
                    pressed={horizontalAlign === "center"}
                    aria-label={t("toolbar.align_center")}
                    onClick={() =>
                      setHorizontalAlign(
                        horizontalAlign === "center" ? "general" : "center",
                      )
                    }
                  />
                  <IconButton
                    icon={<AlignRight />}
                    pressed={horizontalAlign === "right"}
                    aria-label={t("toolbar.align_right")}
                    onClick={() =>
                      setHorizontalAlign(
                        horizontalAlign === "right" ? "general" : "right",
                      )
                    }
                  />
                </div>
              </div>
              <div className="ic-edit-style-subrow">
                <span className="ic-edit-style-sublabel">
                  {t("named_styles.vertical_align_label")}
                </span>
                <div className="ic-edit-style-button-group">
                  <IconButton
                    icon={<ArrowUpToLine />}
                    pressed={verticalAlign === "top"}
                    aria-label={t("toolbar.vertical_align_top")}
                    onClick={() => setVerticalAlign("top")}
                  />
                  <IconButton
                    icon={<ArrowMiddleFromLine />}
                    pressed={verticalAlign === "center"}
                    aria-label={t("toolbar.vertical_align_middle")}
                    onClick={() => setVerticalAlign("center")}
                  />
                  <IconButton
                    icon={<ArrowDownToLine />}
                    pressed={verticalAlign === "bottom"}
                    aria-label={t("toolbar.vertical_align_bottom")}
                    onClick={() => setVerticalAlign("bottom")}
                  />
                </div>
              </div>
            </>
          )}
        </div>
        <div className="ic-edit-style-styled-box ic-edit-style-format-group">
          <Toggle
            checked={includeBorder}
            onChange={setIncludeBorder}
            label={t("named_styles.border_label")}
          />
        </div>
      </div>
      <div className="ic-edit-style-footer">
        <Button variant="secondary" onClick={onClose}>
          {t("named_styles.cancel")}
        </Button>
        <Button startIcon={<Check />} disabled={hasError} onClick={handleSave}>
          {t("named_styles.apply")}
        </Button>
      </div>
    </div>
  );
};

export default EditNamedStyle;
