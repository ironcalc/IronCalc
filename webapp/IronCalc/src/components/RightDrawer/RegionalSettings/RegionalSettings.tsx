import styled from "@emotion/styled";
import { getAllTimezones, getSupportedLocales } from "@ironcalc/wasm";
import {
  Autocomplete,
  type AutocompleteProps,
  Box,
  Button,
  FormControl,
  FormHelperText,
  MenuItem,
  Select,
  TextField,
} from "@mui/material";
import { Check, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { theme } from "../../../theme";

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

// Derive supported languages from localeDisplayNames keys
const SUPPORTED_LANGUAGES = Object.keys(localeDisplayNames);

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
    <Container>
      <Header>
        <HeaderTitle>{t("regional_settings.title")}</HeaderTitle>
        <IconButtonWrapper
          onClick={properties.onClose}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              properties.onClose();
            }
          }}
          aria-label={t("right_drawer.close")}
          tabIndex={0}
        >
          <X />
        </IconButtonWrapper>
      </Header>

      <Content
        onClick={(event) => event.stopPropagation()}
        onMouseDown={(event) => event.stopPropagation()}
      >
        <FormSection>
          <StyledSectionTitle>
            {t("regional_settings.locale.title")}
          </StyledSectionTitle>
          <FieldWrapper>
            <StyledLabel htmlFor="locale">
              {t("regional_settings.locale.locale_label")}
            </StyledLabel>
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
                    sx: menuPaperStyles,
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
              <HelperBox>
                <Row>
                  {t("regional_settings.locale.locale_example1")}
                  <RowValue>1,234.56</RowValue>
                </Row>
                <Row>
                  {t("regional_settings.locale.locale_example2")}
                  <RowValue>12/31/2025 09:21:06 PM</RowValue>
                </Row>
                <Row>
                  {t("regional_settings.locale.locale_example3")}
                  <RowValue>Semicolon (;)</RowValue>
                </Row>
              </HelperBox>
            </FormControl>
          </FieldWrapper>
        </FormSection>
        <FormSection>
          <StyledSectionTitle>
            {t("regional_settings.language.title")}
          </StyledSectionTitle>
          <FieldWrapper>
            <StyledLabel htmlFor="language">
              {t("regional_settings.language.language_label")}
            </StyledLabel>
            <FormControl fullWidth>
              <StyledSelect
                id="language"
                value={selectedLanguage}
                onChange={(event) => {
                  setSelectedLanguage(event.target.value as string);
                }}
                MenuProps={{
                  PaperProps: {
                    sx: menuPaperStyles,
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
                {SUPPORTED_LANGUAGES.map((lang) => (
                  <StyledMenuItem key={lang} value={lang}>
                    {t(`regional_settings.language.display_language.${lang}`)}
                    {lang !== "en" && (
                      <SecondaryText>
                        (
                        {t(
                          `regional_settings.language.display_language_current_lang.${lang}`,
                        )}
                        )
                      </SecondaryText>
                    )}
                  </StyledMenuItem>
                ))}
              </StyledSelect>
              <StyledHelperText>
                {t("regional_settings.language.language_helper")}
              </StyledHelperText>
            </FormControl>
          </FieldWrapper>
        </FormSection>
        <FormSection>
          <StyledSectionTitle>
            {t("regional_settings.timezone.title")}
          </StyledSectionTitle>
          <FieldWrapper>
            <StyledLabel htmlFor="timezone">
              {t("regional_settings.timezone.timezone_label")}
            </StyledLabel>
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
                    sx: { ...menuPaperStyles, margin: "4px 0px" },
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
              <StyledHelperText>
                {t("regional_settings.timezone.timezone_helper")}
              </StyledHelperText>
            </FormControl>
          </FieldWrapper>
        </FormSection>
      </Content>

      <Footer>
        <SaveButton
          variant="contained"
          disableElevation
          startIcon={<Check size={16} />}
          onClick={handleSave}
        >
          {t("num_fmt.save")}
        </SaveButton>
      </Footer>
    </Container>
  );
};

const Container = styled("div")({
  height: "100%",
  display: "flex",
  flexDirection: "column",
});

const Header = styled("div")({
  height: "40px",
  display: "flex",
  alignItems: "center",
  justifyContent: "flex-end",
  padding: "0 8px",
  borderBottom: `1px solid ${theme.palette.grey[300]}`,
});

const HeaderTitle = styled("div")({
  width: "100%",
  fontSize: "12px",
});

const IconButtonWrapper = styled("div")`
  &:hover {
    background-color: ${theme.palette.grey["50"]};
  }
  display: flex;
  border-radius: 4px;
  height: 24px;
  width: 24px;
  cursor: pointer;
  align-items: center;
  justify-content: center;
  svg {
    width: 16px;
    height: 16px;
    stroke-width: 1.5;
  }
`;

const Content = styled("div")({
  flex: 1,
  display: "flex",
  flexDirection: "column",
  fontSize: "12px",
  overflow: "auto",
});

const FormSection = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px 12px;
  border-bottom: 1px solid ${theme.palette.grey[300]};
  &:last-child {
    border-bottom: none;
  }
`;

const StyledSectionTitle = styled("h1")`
  font-size: 14px;
  font-weight: 600;
  font-family: Inter;
  margin: 0px;
  color: ${theme.palette.text.primary};
`;

const StyledSelect = styled(Select)`
  font-size: 12px;
  height: 32px;
  & .MuiInputBase-root {
    padding: 0px !important;
  }
  & .MuiInputBase-input {
    font-size: 12px;
    height: 20px;
    padding-right: 0px !important;
    margin: 0px;
  }
  & .MuiSelect-select {
    padding: 8px 32px 8px 8px !important;
    font-size: 12px;
  }
  & .MuiSvgIcon-root {
    right: 4px !important;
  }
`;

const StyledHelperText = styled(FormHelperText)(() => ({
  fontSize: "12px",
  fontFamily: "Inter",
  color: theme.palette.grey[500],
  margin: 0,
  marginTop: "6px",
  padding: 0,
  lineHeight: 1.4,
}));

const HelperBox = styled("div")`
  display: flex;
  flex-direction: column;
  align-items: start;
  justify-content: center;
  gap: 2px;
  box-sizing: border-box;
  border: 1px solid ${theme.palette.grey["300"]};
  font-family: Inter;
  width: 100%;
  height: 100%;
  margin-top: 8px;
  background-color: ${theme.palette.grey["100"]};
  border-radius: 4px;
  padding: 8px;
`;

const Row = styled("div")`
  display: flex;
  flex-direction: row;
  gap: 4px;
  width: 100%;
  justify-content: space-between;
  color: ${theme.palette.grey[700]};
`;

const RowValue = styled("span")`
  font-size: 12px;
  font-family: Inter;
  font-weight: normal;
  color: ${theme.palette.grey[500]};
`;

const SecondaryText = styled("span")`
  color: ${theme.palette.grey[500]};
  margin-left: 4px;
`;

// Autocomplete with customized styles
// Value => string,
// multiple => false, (we cannot select multiple timezones)
// disableClearable => true, (the timezone must always have a value)
// freeSolo => false (the timezone must be from the list)
type TimezoneAutocompleteProps = AutocompleteProps<string, false, true, false>;
const StyledAutocomplete = styled((props: TimezoneAutocompleteProps) => (
  <Autocomplete<string, false, true, false> {...props} />
))`
  & .MuiInputBase-root {
    padding: 0px !important;
    height: 32px;
  }
  & .MuiInputBase-input {
    font-size: 12px;
    height: 20px;
    padding: 0px;
    padding-right: 0px !important;
    margin: 0px;
  }
  & .MuiAutocomplete-popupIndicator:hover {
    background-color: transparent !important;
  }
  & .MuiAutocomplete-popupIndicator {
    & .MuiTouchRipple-root {
      display: none;
    }
  }
  & .MuiOutlinedInput-root .MuiAutocomplete-endAdornment {
    right: 4px;
  }
  & .MuiOutlinedInput-root .MuiAutocomplete-input {
    padding: 8px !important;
  }
`;

const menuPaperStyles = {
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
};

const StyledMenuItem = styled(MenuItem)`
  padding: 8px !important;
  height: 32px !important;
  min-height: 32px !important;
  border-radius: 4px;
  display: flex;
  align-items: center;
  font-size: 12px;

  &.Mui-selected {
    background-color: ${theme.palette.grey[50]} !important;
  }

  &.Mui-selected:hover {
    background-color: ${theme.palette.grey[50]} !important;
  }

  &:hover {
    background-color: ${theme.palette.grey[50]} !important;
  }
`;

const FieldWrapper = styled(Box)`
  display: flex;
  flex-direction: column;
  width: 100%;
  gap: 6px;
`;

const StyledLabel = styled("label")`
  font-size: 12px;
  font-family: "Inter";
  font-weight: 500;
  color: ${theme.palette.text.primary};
  display: block;
`;

const Footer = styled("div")`
  color: ${theme.palette.grey[700]};
  display: flex;
  align-items: center;
  border-top: 1px solid ${theme.palette.grey["300"]};
  font-family: Inter;
  justify-content: flex-end;
  padding: 8px;
  gap: 8px;
`;

const SaveButton = styled(Button)`
  text-transform: none;
  min-width: fit-content;
  font-size: 12px;
`;

export default RegionalSettings;
