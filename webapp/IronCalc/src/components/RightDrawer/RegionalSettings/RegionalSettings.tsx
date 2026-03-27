import { getAllTimezones, getSupportedLocales } from "@ironcalc/wasm";
import {
  Autocomplete,
  type AutocompleteProps,
  FormControl,
  MenuItem,
  Select,
  styled,
  TextField,
} from "@mui/material";
import type { Theme } from "@mui/material/styles";
import { Check, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import { IconButton } from "../../Button/IconButton";
import "./regional-settings.css";

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

      {/** biome-ignore lint/a11y/noStaticElementInteractions: mouse-driven resize handle for drawer; not keyboard-accessible yet */}
      {/** biome-ignore lint/a11y/useKeyWithClickEvents: mouse-driven resize handle for drawer; not keyboard-accessible yet */}
      <div
        className="ic-regional-settings-content"
        onClick={(event) => event.stopPropagation()}
        onMouseDown={(event) => event.stopPropagation()}
      >
        <div className="ic-regional-settings-section">
          <div className="ic-regional-settings-section-title">
            {t("regional_settings.locale.title")}
          </div>
          <div className="ic-regional-settings-field-wrapper">
            <label className="ic-regional-settings-label" htmlFor="locale">
              {t("regional_settings.locale.locale_label")}
            </label>
            <FormControl fullWidth>
              <StyledSelect
                id="locale"
                value={selectedLocale}
                onChange={(event) => {
                  setSelectedLocale(event.target.value as string);
                }}
                renderValue={(value) => getLocaleDisplayName(value as string)}
                MenuProps={{
                  PaperProps: {
                    sx: (theme) => menuPaperStyles(theme),
                  },
                  TransitionProps: {
                    timeout: 0,
                  },
                  anchorOrigin: {
                    vertical: "bottom",
                    horizontal: "center",
                  },
                  transformOrigin: {
                    vertical: "top",
                    horizontal: "center",
                  },
                  marginThreshold: 0,
                }}
              >
                {locales.map((locale) => (
                  <StyledMenuItem key={locale} value={locale}>
                    {getLocaleDisplayName(locale)}
                  </StyledMenuItem>
                ))}
              </StyledSelect>
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
            </FormControl>
          </div>
        </div>
        <div className="ic-regional-settings-section">
          <h1 className="ic-regional-settings-section-title">
            {t("regional_settings.timezone.title")}
          </h1>
          <div className="ic-regional-settings-field-wrapper">
            <label className="ic-regional-settings-label" htmlFor="timezone">
              {t("regional_settings.timezone.timezone_label")}
            </label>
            <FormControl fullWidth>
              <StyledAutocomplete
                id="timezone"
                value={selectedTimezone}
                onChange={(_event, newValue) => {
                  setSelectedTimezone(newValue);
                }}
                options={timezones}
                renderInput={(params) => <TextField {...params} />}
                renderOption={(props, option) => (
                  <StyledMenuItem {...props} key={option as string}>
                    {option as string}
                  </StyledMenuItem>
                )}
                disableClearable
                slotProps={{
                  paper: {
                    sx: (theme) => ({
                      ...menuPaperStyles(theme),
                      margin: "4px 0px",
                    }),
                  },
                  popper: {
                    sx: {
                      "& .MuiAutocomplete-paper": {
                        transition: "none !important",
                      },
                    },
                  },
                  popupIndicator: {
                    disableRipple: true,
                  },
                }}
              />
              <p className="ic-regional-settings-helper-text">
                {t("regional_settings.timezone.timezone_helper")}
              </p>
            </FormControl>
          </div>
        </div>
      </div>

      <div className="ic-regional-settings-footer">
        <Button startIcon={<Check />} onClick={handleSave}>
          {t("num_fmt.save")}
        </Button>
      </div>
    </div>
  );
};

const StyledSelect = styled(Select)({
  fontSize: 12,
  height: 32,

  "& .MuiInputBase-root": {
    padding: "0px !important",
  },

  "& .MuiInputBase-input": {
    fontSize: 12,
    height: 20,
    paddingRight: "0px !important",
    margin: 0,
  },

  "& .MuiSelect-select": {
    padding: "8px 32px 8px 8px !important",
    fontSize: 12,
  },

  "& .MuiSvgIcon-root": {
    right: "4px !important",
  },
});

// Autocomplete with customized styles
// Value => string,
// multiple => false, (we cannot select multiple timezones)
// disableClearable => true, (the timezone must always have a value)
// freeSolo => false (the timezone must be from the list)
type TimezoneAutocompleteProps = AutocompleteProps<string, false, true, false>;
const StyledAutocomplete = styled((props: TimezoneAutocompleteProps) => (
  <Autocomplete<string, false, true, false> {...props} />
))({
  "& .MuiInputBase-root": {
    padding: "0px !important",
    height: 32,
  },

  "& .MuiInputBase-input": {
    fontSize: 12,
    height: 20,
    padding: 0,
    paddingRight: "0px !important",
    margin: 0,
  },

  "& .MuiAutocomplete-popupIndicator:hover": {
    backgroundColor: "transparent !important",
  },

  "& .MuiAutocomplete-popupIndicator": {
    "& .MuiTouchRipple-root": {
      display: "none",
    },
  },

  "& .MuiOutlinedInput-root .MuiAutocomplete-endAdornment": {
    right: 4,
  },

  "& .MuiOutlinedInput-root .MuiAutocomplete-input": {
    padding: "8px !important",
  },
});

const menuPaperStyles = (theme: Theme) => ({
  boxSizing: "border-box",
  marginTop: "4px",
  padding: "4px",
  borderRadius: "8px",
  transition: "none !important",
  "& .MuiList-padding": {
    padding: 0,
  },
  "& .MuiList-root": {
    padding: 0,
  },
  "& .MuiAutocomplete-noOptions": {
    padding: "8px",
    fontSize: "12px",
    fontFamily: "Inter",
  },
  "& .MuiMenuItem-root": {
    height: "32px !important",
    padding: "8px !important",
    minHeight: "32px !important",
  },
  "& .MuiAutocomplete-option[aria-selected='true']": {
    backgroundColor: `${theme.palette.grey[100]} !important`,
    fontWeight: "500 !important",
  },
});

const StyledMenuItem = styled(MenuItem)(({ theme }) => ({
  padding: "8px !important",
  height: "32px !important",
  minHeight: "32px !important",
  borderRadius: 4,
  display: "flex",
  alignItems: "center",
  fontSize: 12,

  "&.Mui-selected": {
    backgroundColor: `${theme.palette.grey[50]} !important`,
  },

  "&.Mui-selected:hover": {
    backgroundColor: `${theme.palette.grey[50]} !important`,
  },

  "&:hover": {
    backgroundColor: `${theme.palette.grey[50]} !important`,
  },
}));

export default RegionalSettings;
