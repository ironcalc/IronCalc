import type { CellStyle, FmtSettings } from "@ironcalc/wasm";
import { Check } from "lucide-react";
import { type CSSProperties, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import { NumberFormats } from "../../FormatMenu/formatUtil";
import { Input } from "../../Input/Input";
import { Select } from "../../Select/Select";
import FormatStylePicker, {
  type FormatStyle,
} from "../ConditionalFormatting/FormatStylePicker";
import "./edit-named-style.css";

export interface SaveError {
  nameError: string;
}

interface EditNamedStyleProps {
  name: string;
  style: CellStyle;
  formatOptions: FmtSettings;
  existingStyleNames: string[];
  onSave: (payload: NamedStyleSavePayload) => SaveError;
  onClose: () => void;
}

export interface NamedStyleSavePayload {
  name: string;
  style: CellStyle;
}

function formatStyleToPreview(formatStyle: FormatStyle): CSSProperties {
  const decorations: string[] = [];
  if (formatStyle.underline) {
    decorations.push("underline");
  }
  if (formatStyle.strike) {
    decorations.push("line-through");
  }
  return {
    backgroundColor: formatStyle.fillColor || undefined,
    color: formatStyle.fontColor || undefined,
    fontWeight: formatStyle.bold ? "bold" : undefined,
    fontStyle: formatStyle.italic ? "italic" : undefined,
    textDecoration: decorations.length > 0 ? decorations.join(" ") : undefined,
  };
}

function initFormatStyle(style: CellStyle): FormatStyle {
  return {
    bold: style.font.b,
    italic: style.font.i,
    underline: style.font.u,
    strike: style.font.strike,
    fontColor: style.font.color ?? "",
    fillColor: style.fill.fg_color ?? "",
  };
}

const CUSTOM_VALUE = "__custom__";

const EditNamedStyle = ({
  name: initialName,
  style,
  formatOptions,
  existingStyleNames,
  onSave,
  onClose,
}: EditNamedStyleProps) => {
  const { t } = useTranslation();

  const getDefaultName = () => {
    if (initialName) {
      return initialName;
    }
    const existing = new Set(existingStyleNames.map((n) => n.toLowerCase()));
    let counter = 1;
    let candidate = `Style${counter}`;
    while (existing.has(candidate.toLowerCase())) {
      counter++;
      candidate = `Style${counter}`;
    }
    return candidate;
  };

  const [name, setName] = useState(getDefaultName);
  const [nameError, setNameError] = useState("");
  const [formatStyle, setFormatStyle] = useState<FormatStyle>(() =>
    initFormatStyle(style),
  );
  const [numFmt, setNumFmt] = useState<string>(style.num_fmt);
  const [customFmt, setCustomFmt] = useState("");

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
  const isCustom = !knownFormats.includes(numFmt);

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
      setNumFmt(customFmt || "");
    } else {
      setNumFmt(value);
    }
  };

  const selectValue = isCustom ? CUSTOM_VALUE : numFmt;
  const hasError = !!nameError || !name.trim();

  const handleSave = () => {
    if (hasError) {
      return;
    }
    const newStyle = {
      ...style,
      num_fmt: numFmt,
      fill: {
        ...style.fill,
        pattern_type: formatStyle.fillColor ? "solid" : "none",
        fg_color: formatStyle.fillColor || undefined,
      },
      font: {
        ...style.font,
        b: formatStyle.bold,
        i: formatStyle.italic,
        u: formatStyle.underline,
        strike: formatStyle.strike,
        color: formatStyle.fontColor || undefined,
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
            style={formatStyleToPreview(formatStyle)}
          >
            Aa
          </div>
          <span className="ic-edit-style-header-box-text">
            {name.trim() || t("named_styles.new_style")}
          </span>
        </div>
        <div className="ic-edit-style-styled-box">
          <Input
            autoFocus
            type="text"
            label="Name"
            placeholder="Style name"
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
              label="Format"
              value={selectValue}
              options={formatSelectOptions}
              onChange={handleFormatChange}
            />
            {isCustom && (
              <Input
                type="text"
                placeholder='e.g. #,##0.00 or "€"#,##0'
                value={customFmt}
                onChange={(e) => {
                  setCustomFmt(e.target.value);
                  setNumFmt(e.target.value);
                }}
                onKeyDown={handleKeyDown}
              />
            )}
          </div>
          <div className="ic-edit-style-format-group">
            <span className="ic-edit-style-label">Style</span>
            <FormatStylePicker value={formatStyle} onChange={setFormatStyle} />
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
