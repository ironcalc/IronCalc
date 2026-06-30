import type {
  CellStyle,
  FmtSettings,
  IronCalcTheme,
  Model,
} from "@ironcalc/wasm";
import { Check } from "lucide-react";
import { type CSSProperties, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import { resolveColorToHex } from "../../ColorPicker/util";
import { NumberFormats } from "../../FormatMenu/formatUtil";
import { Input } from "../../Input/Input";
import { Select } from "../../Select/Select";
import FormatStylePicker, {
  type FormatStyle,
} from "../ConditionalFormatting/FormatStylePicker";
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
}

function formatStyleToPreview(
  formatStyle: FormatStyle,
  currentTheme: IronCalcTheme,
): CSSProperties {
  const decorations: string[] = [];
  if (formatStyle.underline) {
    decorations.push("underline");
  }
  if (formatStyle.strike) {
    decorations.push("line-through");
  }
  return {
    backgroundColor:
      resolveColorToHex(formatStyle.fillColor, currentTheme) || undefined,
    color: resolveColorToHex(formatStyle.fontColor, currentTheme) || undefined,
    fontWeight: formatStyle.bold ? "bold" : undefined,
    fontStyle: formatStyle.italic ? "italic" : undefined,
    textDecoration: decorations.length > 0 ? decorations.join(" ") : undefined,
  };
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
  const customFmtInputRef = useRef<HTMLInputElement>(null);

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
  const customFmtError = isCustom && !customFmt.trim();
  const hasError = !!nameError || !name.trim() || customFmtError;

  const handleSave = () => {
    if (hasError) {
      setCustomFmtTouched(true);
      return;
    }
    const newStyle = {
      ...style,
      num_fmt: numFmt,
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
    const error = onSave({ name: name.trim(), style: newStyle });
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
            style={formatStyleToPreview(formatStyle, currentTheme)}
          >
            {getPreviewText(numFmt, formatOptions, t)}
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
          <div className="ic-edit-style-format-group">
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
          </div>
          <div className="ic-edit-style-format-group">
            <span className="ic-edit-style-label">
              {t("named_styles.style_label")}
            </span>
            <FormatStylePicker
              value={formatStyle}
              onChange={setFormatStyle}
              currentTheme={currentTheme}
            />
          </div>
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
