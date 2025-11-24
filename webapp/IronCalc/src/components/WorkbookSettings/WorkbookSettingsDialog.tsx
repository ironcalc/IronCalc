import styled from "@emotion/styled";
import {
  Autocomplete,
  Box,
  Dialog,
  FormControl,
  MenuItem,
  Select,
  TextField,
} from "@mui/material";
import { Check, X } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { theme } from "../../theme";

type WorkbookSettingsDialogProps = {
  open: boolean;
  onClose: () => void;
  initialLocale?: string;
  initialTimezone?: string;
  onSave?: (locale: string, timezone: string) => void;
};

const WorkbookSettingsDialog = (properties: WorkbookSettingsDialogProps) => {
  const { t } = useTranslation();
  const [selectedLocale, setSelectedLocale] = useState<string>(
    properties.initialLocale || "",
  );
  const [selectedTimezone, setSelectedTimezone] = useState<string>(
    properties.initialTimezone || "",
  );

  const handleSave = () => {
    if (properties.onSave && selectedLocale && selectedTimezone) {
      properties.onSave(selectedLocale, selectedTimezone);
    }
    properties.onClose();
  };

  const timezones = [
    "Berlin, Germany (GMT+1)",
    "New York, USA (GMT-5)",
    "Tokyo, Japan (GMT+9)",
    "London, UK (GMT+0)",
    "Sydney, Australia (GMT+10)",
  ];

  const locales = ["en-US", "en-GB", "de-DE", "fr-FR", "es-ES"];

  return (
    <StyledDialog
      open={properties.open}
      onClose={(_event, reason) => {
        if (reason === "backdropClick" || reason === "escapeKeyDown") {
          properties.onClose();
        }
      }}
    >
      <StyledDialogTitle>
        {t("workbook_settings.title")}
        <Cross
          onClick={properties.onClose}
          tabIndex={0}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              properties.onClose();
            }
          }}
        >
          <X />
        </Cross>
      </StyledDialogTitle>

      <StyledDialogContent
        onClick={(event) => event.stopPropagation()}
        onMouseDown={(event) => event.stopPropagation()}
      >
        <StyledSectionTitle>
          {t("workbook_settings.locale_and_timezone.title")}
        </StyledSectionTitle>
        <FieldWrapper>
          <StyledLabel htmlFor="locale">
            {t("workbook_settings.locale_and_timezone.locale_label")}
          </StyledLabel>
          <FormControl fullWidth>
            <StyledSelect
              id="locale"
              value={selectedLocale}
              onChange={(event) => {
                setSelectedLocale(event.target.value as string);
              }}
              displayEmpty
              MenuProps={{
                PaperProps: {
                  sx: menuPaperStyles,
                },
                TransitionProps: {
                  timeout: 0,
                },
              }}
            >
              {locales.map((locale) => {
                const isSelected = locale === selectedLocale;
                return (
                  <StyledMenuItem
                    key={locale}
                    value={locale}
                    $isSelected={isSelected}
                  >
                    {locale}
                  </StyledMenuItem>
                );
              })}
            </StyledSelect>
            <HelperBox>
              <Row>
                {t("workbook_settings.locale_and_timezone.locale_example1")}
                <RowValue>1,234.56</RowValue>
              </Row>
              <Row>
                {t("workbook_settings.locale_and_timezone.locale_example2")}
                <RowValue>12/31/2025</RowValue>
              </Row>
              <Row>
                {t("workbook_settings.locale_and_timezone.locale_example3")}
                <RowValue>11/23/2025 09:21:06 PM</RowValue>
              </Row>
              <Row>
                {t("workbook_settings.locale_and_timezone.locale_example4")}
                <RowValue>Monday</RowValue>
              </Row>
            </HelperBox>
          </FormControl>
        </FieldWrapper>
        <FieldWrapper>
          <StyledLabel htmlFor="timezone">
            {t("workbook_settings.locale_and_timezone.timezone_label")}
          </StyledLabel>
          <FormControl fullWidth>
            <StyledAutocomplete
              id="timezone"
              value={selectedTimezone}
              onChange={(_event, newValue) => {
                setSelectedTimezone((newValue as string) || "");
              }}
              options={timezones}
              renderInput={(params) => <TextField {...params} />}
              renderOption={(props, option) => {
                const isSelected = option === selectedTimezone;
                return (
                  <StyledMenuItem
                    {...props}
                    key={option as string}
                    $isSelected={isSelected}
                  >
                    {option as string}
                  </StyledMenuItem>
                );
              }}
              disableClearable
              slotProps={{
                paper: {
                  sx: menuPaperStyles,
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
            <HelperBox>
              <Row>
                {t("workbook_settings.locale_and_timezone.timezone_example1")}
                <RowValue>23/11/2025</RowValue>
              </Row>
              <Row>
                {t("workbook_settings.locale_and_timezone.timezone_example2")}
                <RowValue>11/23/2025 09:21:06 PM</RowValue>
              </Row>
            </HelperBox>
          </FormControl>
        </FieldWrapper>
      </StyledDialogContent>

      <DialogFooter>
        <StyledButton onClick={handleSave} tabIndex={0}>
          <Check
            style={{ width: "16px", height: "16px", marginRight: "8px" }}
          />
          {t("num_fmt.save")}
        </StyledButton>
      </DialogFooter>
    </StyledDialog>
  );
};

const StyledDialog = styled(Dialog)`
  & .MuiPaper-root {
    max-width: 320px;
    min-width: 320px;
    border-radius: 8px;
    padding: 0px;
  }
`;

const StyledDialogTitle = styled("div")`
  display: flex;
  align-items: center;
  height: 44px;
  font-size: 14px;
  font-weight: 500;
  font-family: Inter;
  padding: 0px 12px;
  justify-content: space-between;
  border-bottom: 1px solid ${theme.palette.grey["300"]};
`;

const Cross = styled("div")`
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

const StyledDialogContent = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 12px;
  font-size: 12px;
  margin: 12px;
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

const StyledAutocomplete = styled(Autocomplete)`
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
};

const StyledMenuItem = styled(MenuItem)<{ $isSelected?: boolean }>`
  padding: 8px !important;
  height: 32px !important;
  min-height: 32px !important;
  border-radius: 4px;
  display: flex;
  align-items: center;
  font-size: 12px;
  background-color: ${({ $isSelected }) =>
    $isSelected ? theme.palette.grey[50] : "transparent"} !important;
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

const DialogFooter = styled("div")`
  color: ${theme.palette.grey[700]};
  display: flex;
  align-items: center;
  border-top: 1px solid ${theme.palette.grey["300"]};
  font-family: Inter;
  justify-content: flex-end;
  padding: 12px;
`;

const StyledButton = styled("div")`
  cursor: pointer;
  color: ${theme.palette.common.white};
  background: ${theme.palette.primary.main};
  padding: 0px 10px;
  height: 36px;
  line-height: 36px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  font-family: "Inter";
  font-size: 14px;
  &:hover {
    background: ${theme.palette.primary.dark};
  }
`;

export default WorkbookSettingsDialog;
