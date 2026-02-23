import styled from "@emotion/styled";
import { getAllTimezones, getSupportedLocales } from "@ironcalc/wasm";
import {
  Autocomplete,
  type AutocompleteProps,
  Box,
  Button,
  FormControl,
  MenuItem,
  Select,
  TextField,
} from "@mui/material";
import { Check, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { theme } from "../../../theme";
import ColorPicker from "../../ColorPicker/ColorPicker";
import {
  Container,
  Content,
  FormSection,
  Header,
  HeaderTitle,
  IconButtonWrapper,
  StyledColorInput,
  StyledHelperText,
  StyledLabel,
  StyledSectionTitle,
  StyledTextField,
} from "../Common";

export interface WorkbookSettings {
  locale: string;
  timezone: string;
  language: string;
  defaultColumnWidth: number;
  defaultRowHeight: number;
  defaultFontSize: number;
  defaultTextColor: string;
  defaultBackgroundColor: string;
}
interface WorkbookSettingsProps {
  onClose: () => void;
  settings: WorkbookSettings;
  onSave: (settings: WorkbookSettings) => void;
}

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

const WorkbookSettingsDrawer = (properties: WorkbookSettingsProps) => {
  const { t } = useTranslation();
  const locales = getSupportedLocales();
  const settings = properties.settings;

  const timezones = getAllTimezones();

  const [selectedLocale, setSelectedLocale] = useState(settings.locale);
  const [selectedTimezone, setSelectedTimezone] = useState(settings.timezone);
  const [selectedLanguage, setSelectedLanguage] = useState(settings.language);

  const [defaultColumnWidth, setDefaultColumnWidth] = useState(
    settings.defaultColumnWidth,
  );
  const [defaultRowHeight, setDefaultRowHeight] = useState(
    settings.defaultRowHeight,
  );
  const [defaultFontSize, setDefaultFontSize] = useState(
    settings.defaultFontSize,
  );
  const [defaultTextColor, setDefaultTextColor] = useState(
    settings.defaultTextColor,
  );
  const [defaultBackgroundColor, setDefaultBackgroundColor] = useState(
    settings.defaultBackgroundColor,
  );

  const [backgroundColorPickerOpen, setBackgroundColorPickerOpen] =
    useState(false);
  const backgroundColorButton = useRef(null);

  const [textColorPickerOpen, setTextColorPickerOpen] = useState(false);
  const textColorButton = useRef(null);

  useEffect(() => {
    setSelectedLocale(settings.locale);
    setSelectedTimezone(settings.timezone);
    setSelectedLanguage(settings.language);
    setDefaultColumnWidth(settings.defaultColumnWidth);
    setDefaultRowHeight(settings.defaultRowHeight);
    setDefaultFontSize(settings.defaultFontSize);
    setDefaultTextColor(settings.defaultTextColor);
    setDefaultBackgroundColor(settings.defaultBackgroundColor);
  }, [
    settings.locale,
    settings.timezone,
    settings.language,
    settings.defaultColumnWidth,
    settings.defaultRowHeight,
    settings.defaultFontSize,
    settings.defaultTextColor,
    settings.defaultBackgroundColor,
  ]);

  const handleSave = () => {
    properties.onSave({
      locale: selectedLocale,
      timezone: selectedTimezone,
      language: selectedLanguage,
      defaultColumnWidth,
      defaultRowHeight,
      defaultFontSize,
      defaultTextColor,
      defaultBackgroundColor,
    });
    properties.onClose();
  };

  return (
    <Container>
      <Header>
        <HeaderTitle>{t("workbook_settings.title")}</HeaderTitle>
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
            {t("workbook_settings.locale.title")}
          </StyledSectionTitle>
          <FieldWrapper>
            <StyledLabel htmlFor="locale">
              {t("workbook_settings.locale.locale_label")}
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
                  {t("workbook_settings.locale.locale_example1")}
                  <RowValue>
                    {localeFormatExamples[selectedLocale]?.number ?? "1,234.56"}
                  </RowValue>
                </Row>
                <Row>
                  {t("workbook_settings.locale.locale_example2")}
                  <RowValue>
                    {localeFormatExamples[selectedLocale]?.dateTime ??
                      "10/17/2026 09:21:06 PM"}
                  </RowValue>
                </Row>
                <Row>
                  {t("workbook_settings.locale.locale_example3")}
                  <RowValue>
                    {(() => {
                      const delimiterType =
                        localeFormatExamples[selectedLocale]?.delimiterType ??
                        "comma";
                      const delimiterChar =
                        localeFormatExamples[selectedLocale]?.delimiterChar ??
                        ",";
                      const delimiterLabel = t(
                        `workbook_settings.locale.delimiter_${delimiterType}`,
                      );
                      return `${delimiterLabel} (${delimiterChar})`;
                    })()}
                  </RowValue>
                </Row>
              </HelperBox>
            </FormControl>
          </FieldWrapper>
        </FormSection>
        <FormSection>
          <StyledSectionTitle>
            {t("workbook_settings.timezone.title")}
          </StyledSectionTitle>
          <FieldWrapper>
            <StyledLabel htmlFor="timezone">
              {t("workbook_settings.timezone.timezone_label")}
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
                {t("workbook_settings.timezone.timezone_helper")}
              </StyledHelperText>
            </FormControl>
          </FieldWrapper>
        </FormSection>
        <FormSection>
          <StyledSectionTitle>
            {t("workbook_settings.defaults.title")}
          </StyledSectionTitle>
          <StyledLabel htmlFor="column-width">
            {t("workbook_settings.defaults.column_width.column_width")}
          </StyledLabel>
          <FormControl fullWidth>
            <StyledTextField
              id="column-width"
              type="number"
              defaultValue={defaultColumnWidth}
              onChange={(e) => {
                setDefaultColumnWidth(parseInt(e.target.value, 10));
              }}
            />
            <StyledHelperText>
              {t("workbook_settings.defaults.column_width.column_width_helper")}
            </StyledHelperText>
          </FormControl>
          <StyledLabel htmlFor="row-height">
            {t("workbook_settings.defaults.row_height.row_height")}
          </StyledLabel>
          <FormControl fullWidth>
            <StyledTextField
              id="row-height"
              type="number"
              defaultValue={defaultRowHeight}
              onChange={(e) => {
                setDefaultRowHeight(parseInt(e.target.value, 10));
              }}
            />
            <StyledHelperText>
              {t("workbook_settings.defaults.row_height.row_height_helper")}
            </StyledHelperText>
          </FormControl>
          <StyledLabel htmlFor="font-size">
            {t("workbook_settings.defaults.font_size.font_size")}
          </StyledLabel>
          <FormControl fullWidth>
            <StyledTextField
              id="font-size"
              type="number"
              defaultValue={defaultFontSize}
              onChange={(e) => {
                setDefaultFontSize(parseInt(e.target.value, 10));
              }}
            />
            <StyledHelperText>
              {t("workbook_settings.defaults.font_size.font_size_helper")}
            </StyledHelperText>
          </FormControl>
          <StyledLabel htmlFor="text-color">
            {t("workbook_settings.defaults.text_color.text_color")}
          </StyledLabel>
          <FormControl fullWidth>
            <StyledColorInput
              onClick={() => setTextColorPickerOpen(true)}
              ref={textColorButton}
              style={{ background: defaultTextColor }}
            />
            <StyledHelperText>
              {t("workbook_settings.defaults.text_color.text_color_helper")}
            </StyledHelperText>
          </FormControl>
          <StyledLabel htmlFor="background-color">
            {t("workbook_settings.defaults.background_color.background_color")}
          </StyledLabel>
          <FormControl fullWidth>
            <StyledColorInput
              onClick={() => setBackgroundColorPickerOpen(true)}
              ref={backgroundColorButton}
              style={{ background: defaultBackgroundColor }}
            />
            <StyledHelperText>
              {t(
                "workbook_settings.defaults.background_color.background_color_helper",
              )}
            </StyledHelperText>
          </FormControl>
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
      <ColorPicker
        color={defaultBackgroundColor}
        defaultColor=""
        title={t("color_picker.default")}
        onChange={(color): void => {
          setDefaultBackgroundColor(color);
          setBackgroundColorPickerOpen(false);
        }}
        onClose={() => {
          setBackgroundColorPickerOpen(false);
        }}
        anchorEl={backgroundColorButton}
        open={backgroundColorPickerOpen}
        anchorOrigin={{
          vertical: "top",
          horizontal: "right",
        }}
        transformOrigin={{
          vertical: "top",
          horizontal: "left",
        }}
      />
      <ColorPicker
        color={defaultTextColor}
        defaultColor="#000000"
        title={t("color_picker.default")}
        onChange={(color): void => {
          setDefaultTextColor(color);
          setTextColorPickerOpen(false);
        }}
        onClose={() => {
          setTextColorPickerOpen(false);
        }}
        anchorEl={textColorButton}
        open={textColorPickerOpen}
        anchorOrigin={{
          vertical: "top",
          horizontal: "right",
        }}
        transformOrigin={{
          vertical: "top",
          horizontal: "left",
        }}
      />
    </Container>
  );
};

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

export default WorkbookSettingsDrawer;
