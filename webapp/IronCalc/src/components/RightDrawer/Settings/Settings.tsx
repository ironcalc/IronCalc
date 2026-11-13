import {
  columnNameFromNumber,
  columnNumberFromName,
  type FocusOnOpen,
  getAllTimezones,
  getSupportedLocales,
  type WorksheetProperties,
} from "@ironcalc/wasm";
import { Check, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import { IconButton } from "../../Button/IconButton";
import "./settings.css";
import { Input } from "../../Input/Input";
import { Select } from "../../Select/Select";
import { Switch } from "../../Switch/Switch";
import { LAST_COLUMN, LAST_ROW } from "../../WorksheetCanvas/constants";

type SettingsProps = {
  onClose: () => void;
  initialLocale: string;
  initialTimezone: string;
  initialLanguage: string;
  // The cell the workbook focuses on when opened, undefined if the setting is off
  initialFocusOnOpen: FocusOnOpen | undefined;
  worksheets: WorksheetProperties[];
  // The cell proposed when the "focus on open" setting is turned on
  getSelectedCell: () => FocusOnOpen;
  onSave: (
    locale: string,
    timezone: string,
    language: string,
    focusOnOpen: FocusOnOpen | null,
  ) => void;
};

// Parses a cell reference like "C5" (or "$C$5")
const parseCellReference = (
  text: string,
): { row: number; column: number } | null => {
  const match = /^\$?([A-Za-z]{1,3})\$?([0-9]{1,7})$/.exec(text.trim());
  if (!match) {
    return null;
  }
  let column: number;
  try {
    column = columnNumberFromName(match[1].toUpperCase());
  } catch {
    return null;
  }
  const row = Number.parseInt(match[2], 10);
  if (row < 1 || row > LAST_ROW || column < 1 || column > LAST_COLUMN) {
    return null;
  }
  return { row, column };
};

// Display mapping for locale codes (e.g., "en" -> "en-US")
const localeDisplayNames: Record<string, string> = {
  en: "en-US",
  es: "es-ES",
  fr: "fr-FR",
  de: "de-DE",
  it: "it-IT",
};

// Locale-specific format examples (independent of display language)
// delimiterType is used to look up the translated word, delimiterChar is the actual character
const localeFormatExamples: Record<
  string,
  {
    number: string;
    dateTime: string;
    delimiterType: "comma" | "semicolon";
    delimiterChar: string;
  }
> = {
  en: {
    number: "1,234.56",
    dateTime: "10/17/2026 09:21:06 PM",
    delimiterType: "comma",
    delimiterChar: ",",
  },
  es: {
    number: "1.234,56",
    dateTime: "17/10/2026 21:21:06",
    delimiterType: "semicolon",
    delimiterChar: ";",
  },
  fr: {
    number: "1 234,56",
    dateTime: "17/10/2026 21:21:06",
    delimiterType: "semicolon",
    delimiterChar: ";",
  },
  de: {
    number: "1.234,56",
    dateTime: "17.10.2026 21:21:06",
    delimiterType: "semicolon",
    delimiterChar: ";",
  },
  it: {
    number: "1.234,56",
    dateTime: "17/10/2026 21:21:06",
    delimiterType: "semicolon",
    delimiterChar: ";",
  },
};

const getLocaleDisplayName = (locale: string): string => {
  return localeDisplayNames[locale] ?? locale;
};

const Settings = (properties: SettingsProps) => {
  const { t } = useTranslation();
  const locales = getSupportedLocales();
  const timezones = getAllTimezones();

  const [selectedLocale, setSelectedLocale] = useState(
    properties.initialLocale,
  );
  const [selectedTimezone, setSelectedTimezone] = useState(
    properties.initialTimezone,
  );
  const [selectedLanguage, setSelectedLanguage] = useState(
    properties.initialLanguage,
  );

  // Focus on open: the sheet (its index) and the cell (as text, like "C5")
  const initialFocus = properties.initialFocusOnOpen;
  const initialFocusEnabled = initialFocus !== undefined;
  const initialFocusSheet = initialFocus?.sheet ?? 0;
  const initialFocusCell = initialFocus
    ? `${columnNameFromNumber(initialFocus.column)}${initialFocus.row}`
    : "A1";
  const [focusEnabled, setFocusEnabled] = useState(initialFocusEnabled);
  const [focusSheet, setFocusSheet] = useState(initialFocusSheet);
  const [focusCell, setFocusCell] = useState(initialFocusCell);

  useEffect(() => {
    setSelectedLocale(properties.initialLocale);
    setSelectedTimezone(properties.initialTimezone);
    setSelectedLanguage(properties.initialLanguage);
  }, [
    properties.initialLocale,
    properties.initialTimezone,
    properties.initialLanguage,
  ]);

  useEffect(() => {
    setFocusEnabled(initialFocusEnabled);
    setFocusSheet(initialFocusSheet);
    setFocusCell(initialFocusCell);
  }, [initialFocusEnabled, initialFocusSheet, initialFocusCell]);

  const handleFocusToggle = (enabled: boolean) => {
    if (enabled) {
      // Propose the cell the user is on
      const { sheet, row, column } = properties.getSelectedCell();
      setFocusSheet(sheet);
      setFocusCell(`${columnNameFromNumber(column)}${row}`);
    }
    setFocusEnabled(enabled);
  };

  const parsedFocusCell = parseCellReference(focusCell);
  const focusCellError = focusEnabled && parsedFocusCell === null;

  // A workbook cannot open on a hidden sheet: only the visible ones are
  // offered (and the current one, if it was hidden after being chosen)
  const focusSheetOptions = properties.worksheets
    .map((worksheet, index) => ({ worksheet, index }))
    .filter(
      ({ worksheet, index }) =>
        worksheet.state === "visible" || index === focusSheet,
    )
    .map(({ worksheet, index }) => ({
      value: `${index}`,
      label: worksheet.name,
      triggerLabel: worksheet.name,
    }));

  const handleSave = () => {
    if (focusCellError) {
      return;
    }
    properties.onSave(
      selectedLocale,
      selectedTimezone,
      selectedLanguage,
      focusEnabled && parsedFocusCell
        ? { sheet: focusSheet, ...parsedFocusCell }
        : null,
    );
    properties.onClose();
  };

  return (
    <div className="ic-settings-container">
      <div className="ic-settings-header">
        <div className="ic-settings-header-title">{t("settings.title")}</div>
        <IconButton
          variant="ghost"
          size="xs"
          icon={<X />}
          onClick={properties.onClose}
          aria-label={t("right_drawer.close")}
        />
      </div>

      <div className="ic-settings-content">
        <div className="ic-settings-section">
          <div className="ic-settings-section-title">
            {t("settings.regional_title")}
          </div>
          <div className="ic-settings-field-wrapper">
            <Select
              label={t("settings.locale.locale_label")}
              value={selectedLocale}
              onChange={setSelectedLocale}
              options={locales.map((locale) => ({
                value: locale,
                label: getLocaleDisplayName(locale),
                triggerLabel: getLocaleDisplayName(locale),
              }))}
            />
            <div className="ic-settings-helper-box">
              <div className="ic-settings-row">
                {t("settings.locale.locale_example1")}
                <span className="ic-settings-row-value">
                  {localeFormatExamples[selectedLocale]?.number ?? "1,234.56"}
                </span>
              </div>
              <div className="ic-settings-row">
                {t("settings.locale.locale_example2")}
                <span className="ic-settings-row-value">
                  {localeFormatExamples[selectedLocale]?.dateTime ??
                    "10/17/2026 09:21:06 PM"}
                </span>
              </div>
              <div className="ic-settings-row">
                {t("settings.locale.locale_example3")}
                <span className="ic-settings-row-value">
                  {(() => {
                    const delimiterType =
                      localeFormatExamples[selectedLocale]?.delimiterType ??
                      "comma";
                    const delimiterChar =
                      localeFormatExamples[selectedLocale]?.delimiterChar ??
                      ",";
                    const delimiterLabel = t(
                      `settings.locale.delimiter_${delimiterType}`,
                    );
                    return `${delimiterLabel} (${delimiterChar})`;
                  })()}
                </span>
              </div>
            </div>
          </div>
          <div className="ic-settings-field-wrapper">
            <Select
              label={t("settings.timezone.timezone_label")}
              helperText={t("settings.timezone.timezone_helper")}
              value={selectedTimezone}
              onChange={setSelectedTimezone}
              options={timezones.map((timezone) => ({
                value: timezone,
                label: timezone,
                triggerLabel: timezone,
              }))}
            />
          </div>
        </div>
        <div className="ic-settings-section">
          <Switch
            className="ic-settings-section-switch"
            label={t("settings.focus_on_open.title")}
            checked={focusEnabled}
            onChange={handleFocusToggle}
          />
          {focusEnabled && (
            <div className="ic-settings-fields">
              <Select
                label={t("settings.focus_on_open.sheet_label")}
                value={`${focusSheet}`}
                onChange={(value) => setFocusSheet(Number.parseInt(value, 10))}
                options={focusSheetOptions}
              />
              <Input
                label={t("settings.focus_on_open.cell_label")}
                value={focusCell}
                error={focusCellError}
                helperText={
                  focusCellError
                    ? t("settings.focus_on_open.invalid_cell")
                    : t("settings.focus_on_open.helper")
                }
                onChange={(event) => setFocusCell(event.target.value)}
                onKeyDown={(event) => event.stopPropagation()}
              />
            </div>
          )}
        </div>
      </div>

      <div className="ic-settings-footer">
        <Button
          startIcon={<Check />}
          onClick={handleSave}
          disabled={focusCellError}
        >
          {t("num_fmt.save")}
        </Button>
      </div>
    </div>
  );
};

export default Settings;
