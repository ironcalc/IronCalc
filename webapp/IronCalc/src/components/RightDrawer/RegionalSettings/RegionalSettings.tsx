import { getAllTimezones, getSupportedLocales } from "@ironcalc/wasm";
import { Check, RotateCcw, X } from "lucide-react";
import { useEffect, useId, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  useColorScheme,
  useInvertColorOptions,
} from "../../../theme/colorScheme";
import {
  DEFAULT_OPTIONS,
  type InvertColorOptions,
} from "../../../theme/invertColor";
import { Button } from "../../Button/Button";
import { IconButton } from "../../Button/IconButton";
import "./regional-settings.css";
import { Select } from "../../Select/Select";

// Sliders for the knobs of the dark-mode color inversion (invertColor.ts).
// Only shown in dark mode; the changes apply live to the whole workbook.
const INVERT_COLOR_SLIDERS: {
  key: keyof InvertColorOptions;
  labelKey: string;
  min: number;
  max: number;
  step: number;
}[] = [
  {
    key: "darkBackgroundL",
    labelKey: "regional_settings.dark_mode.dark_background",
    min: 0,
    max: 1,
    step: 0.005,
  },
  {
    key: "darkForegroundL",
    labelKey: "regional_settings.dark_mode.dark_foreground",
    min: 0,
    max: 1,
    step: 0.005,
  },
  {
    key: "lightnessGamma",
    labelKey: "regional_settings.dark_mode.lightness_curve",
    min: 0.25,
    max: 3,
    step: 0.05,
  },
  {
    key: "chromaGamma",
    labelKey: "regional_settings.dark_mode.chroma_curve",
    min: 0.25,
    max: 3,
    step: 0.05,
  },
];

const DarkModeTuning = () => {
  const { t } = useTranslation();
  const { options, setOptions } = useInvertColorOptions();
  const id = useId();
  const isDefault = INVERT_COLOR_SLIDERS.every(
    ({ key }) => options[key] === DEFAULT_OPTIONS[key],
  );

  return (
    <div className="ic-regional-settings-section">
      <div className="ic-regional-settings-section-title">
        {t("regional_settings.dark_mode.title")}
      </div>
      <div className="ic-regional-settings-sliders">
        {INVERT_COLOR_SLIDERS.map(({ key, labelKey, min, max, step }) => (
          <div key={key} className="ic-regional-settings-slider">
            <label
              htmlFor={`${id}-${key}`}
              className="ic-regional-settings-row"
            >
              {t(labelKey)}
              <span className="ic-regional-settings-row-value">
                {options[key].toFixed(3)}
              </span>
            </label>
            <input
              id={`${id}-${key}`}
              type="range"
              className="ic-regional-settings-range"
              min={min}
              max={max}
              step={step}
              defaultValue={options[key]}
              // Safari does not focus a range input on click; the workbook only
              // leaves focus alone while the active element is in the drawer
              onPointerDown={(e) => e.currentTarget.focus()}
              onChange={(e) =>
                setOptions({ ...options, [key]: Number(e.target.value) })
              }
            />
          </div>
        ))}
      </div>
      <div>
        <Button
          variant="secondary"
          size="sm"
          startIcon={<RotateCcw />}
          disabled={isDefault}
          onClick={() => setOptions(DEFAULT_OPTIONS)}
        >
          {t("regional_settings.dark_mode.reset")}
        </Button>
      </div>
    </div>
  );
};

type RegionalSettingsProps = {
  onClose: () => void;
  initialLocale: string;
  initialTimezone: string;
  initialLanguage: string;
  onSave: (locale: string, timezone: string, language: string) => void;
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

export const getLocaleDisplayName = (locale: string): string => {
  return localeDisplayNames[locale] ?? locale;
};

const RegionalSettings = (properties: RegionalSettingsProps) => {
  const { t } = useTranslation();
  const colorScheme = useColorScheme();
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

  useEffect(() => {
    setSelectedLocale(properties.initialLocale);
    setSelectedTimezone(properties.initialTimezone);
    setSelectedLanguage(properties.initialLanguage);
  }, [
    properties.initialLocale,
    properties.initialTimezone,
    properties.initialLanguage,
  ]);

  const handleSave = () => {
    properties.onSave(selectedLocale, selectedTimezone, selectedLanguage);
    properties.onClose();
  };

  return (
    <div className="ic-regional-settings-container">
      <div className="ic-regional-settings-header">
        <div className="ic-regional-settings-header-title">
          {t("regional_settings.title")}
        </div>
        <IconButton
          variant="ghost"
          size="xs"
          icon={<X />}
          onClick={properties.onClose}
          aria-label={t("right_drawer.close")}
        />
      </div>

      <div className="ic-regional-settings-content">
        <div className="ic-regional-settings-section">
          <div className="ic-regional-settings-section-title">
            {t("regional_settings.locale.title")}
          </div>
          <div className="ic-regional-settings-field-wrapper">
            <Select
              label={t("regional_settings.locale.locale_label")}
              value={selectedLocale}
              onChange={setSelectedLocale}
              options={locales.map((locale) => ({
                value: locale,
                label: getLocaleDisplayName(locale),
                triggerLabel: getLocaleDisplayName(locale),
              }))}
            />
            <div className="ic-regional-settings-helper-box">
              <div className="ic-regional-settings-row">
                {t("regional_settings.locale.locale_example1")}
                <span className="ic-regional-settings-row-value">
                  {localeFormatExamples[selectedLocale]?.number ?? "1,234.56"}
                </span>
              </div>
              <div className="ic-regional-settings-row">
                {t("regional_settings.locale.locale_example2")}
                <span className="ic-regional-settings-row-value">
                  {localeFormatExamples[selectedLocale]?.dateTime ??
                    "10/17/2026 09:21:06 PM"}
                </span>
              </div>
              <div className="ic-regional-settings-row">
                {t("regional_settings.locale.locale_example3")}
                <span className="ic-regional-settings-row-value">
                  {(() => {
                    const delimiterType =
                      localeFormatExamples[selectedLocale]?.delimiterType ??
                      "comma";
                    const delimiterChar =
                      localeFormatExamples[selectedLocale]?.delimiterChar ??
                      ",";
                    const delimiterLabel = t(
                      `regional_settings.locale.delimiter_${delimiterType}`,
                    );
                    return `${delimiterLabel} (${delimiterChar})`;
                  })()}
                </span>
              </div>
            </div>
          </div>
        </div>
        <div className="ic-regional-settings-section">
          <div className="ic-regional-settings-section-title">
            {t("regional_settings.timezone.title")}
          </div>
          <div className="ic-regional-settings-field-wrapper">
            <Select
              label={t("regional_settings.timezone.timezone_label")}
              helperText={t("regional_settings.timezone.timezone_helper")}
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
        {colorScheme === "dark" && <DarkModeTuning />}
      </div>

      <div className="ic-regional-settings-footer">
        <Button startIcon={<Check />} onClick={handleSave}>
          {t("num_fmt.save")}
        </Button>
      </div>
    </div>
  );
};

export default RegionalSettings;
