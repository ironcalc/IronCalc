import styled from "@emotion/styled";
import { FormControl, Switch } from "@mui/material";
import { X } from "lucide-react";
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

export type WorksheetSettings = {
  defaultColumnWidth: number;
  defaultRowHeight: number;
  defaultFontSize: number;
  defaultTextColor: string;
  defaultBackgroundColor: string;
  useWorkbook: boolean;
};

export interface onSaveFunctions {
  onColumnWidthChange: (width: number) => void;
  onRowHeightChange: (height: number) => void;
  onFontSizeChange: (size: number) => void;
  onTextColorChange: (color: string) => void;
  onBackgroundColorChange: (color: string) => void;
  onUseWorkbookChange: (useWorkbook: boolean) => void;
}

interface WorksheetSettingsDrawerProps {
  onClose: () => void;
  settings: WorksheetSettings;
  save: onSaveFunctions;
}

// We want to trigger the change on when:
// 1. The user finishes typing and leaves the input (blur event)
// 2. user clicks up/down arrows
// Not when:
// 1. User is typing
function isEditValue(
  e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement, Element>,
): boolean {
  const native = e.nativeEvent;
  if (native instanceof InputEvent) {
    const inputType = native.inputType;
    if (
      [
        "insertText",
        "deleteContentBackward",
        "deleteContentForward",
        "insertFromPaste",
      ].includes(inputType)
    ) {
      return false;
    }
  }
  return true;
}

const WorksheetSettingsDrawer = (properties: WorksheetSettingsDrawerProps) => {
  const { t } = useTranslation();
  const settings = properties.settings;

  const [useWorkbook, setUseWorkbook] = useState(settings.useWorkbook);
  const [localColumnWidth, setLocalColumnWidth] = useState(
    settings.defaultColumnWidth,
  );
  const [localRowHeight, setLocalRowHeight] = useState(
    settings.defaultRowHeight,
  );
  const [localFontSize, setLocalFontSize] = useState(settings.defaultFontSize);
  const [localTextColor, setLocalTextColor] = useState(
    settings.defaultTextColor,
  );
  const [localBackgroundColor, setLocalBackgroundColor] = useState(
    settings.defaultBackgroundColor,
  );

  const [backgroundColorPickerOpen, setBackgroundColorPickerOpen] =
    useState(false);
  const backgroundColorButton = useRef(null);

  const [textColorPickerOpen, setTextColorPickerOpen] = useState(false);
  const textColorButton = useRef(null);

  useEffect(() => {
    setUseWorkbook(settings.useWorkbook);
    setLocalColumnWidth(settings.defaultColumnWidth);
    setLocalRowHeight(settings.defaultRowHeight);
    setLocalFontSize(settings.defaultFontSize);
    setLocalTextColor(settings.defaultTextColor);
    setLocalBackgroundColor(settings.defaultBackgroundColor);
  }, [
    settings.useWorkbook,
    settings.defaultBackgroundColor,
    settings.defaultColumnWidth,
    settings.defaultFontSize,
    settings.defaultRowHeight,
    settings.defaultTextColor,
  ]);

  return (
    <Container>
      <Header>
        <HeaderTitle>{t("worksheet_settings.title")}</HeaderTitle>
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
      <Content>
        <FormSection>
          <StyledSectionTitle>
            {t("worksheet_settings.defaults.title")}
          </StyledSectionTitle>
          <ToggleBox>
            <ToggleText>
              {t("worksheet_settings.defaults.use_workbook_defaults")}
            </ToggleText>

            <Switch
              checked={useWorkbook}
              onChange={(_e, checked) => {
                setUseWorkbook(checked);
                properties.save.onUseWorkbookChange(checked);
              }}
              slotProps={{
                input: {
                  "aria-label": t(
                    "worksheet_settings.defaults.use_workbook_defaults",
                  ),
                },
              }}
            />
          </ToggleBox>
          {!useWorkbook && (
            <>
              <StyledLabel>
                {t("worksheet_settings.defaults.column_width.column_width")}
              </StyledLabel>
              <FormControl fullWidth>
                <StyledTextField
                  type="number"
                  value={localColumnWidth}
                  onBlur={(e) => {
                    const value = parseInt(e.currentTarget.value, 10);
                    if (!Number.isNaN(value)) {
                      properties.save.onColumnWidthChange(value);
                    }
                  }}
                  onChange={(e) => {
                    const value = parseInt(e.currentTarget.value, 10);
                    if (Number.isNaN(value)) {
                      return;
                    }
                    setLocalColumnWidth(value);
                    if (!isEditValue(e)) {
                      return;
                    }
                    properties.save.onColumnWidthChange(value);
                  }}
                />
                <StyledHelperText>
                  {t(
                    "worksheet_settings.defaults.column_width.column_width_helper",
                  )}
                </StyledHelperText>
              </FormControl>

              <StyledLabel>
                {t("worksheet_settings.defaults.row_height.row_height")}
              </StyledLabel>
              <FormControl fullWidth>
                <StyledTextField
                  type="number"
                  value={localRowHeight}
                  onBlur={(e) => {
                    const value = parseInt(e.currentTarget.value, 10);
                    if (!Number.isNaN(value)) {
                      properties.save.onRowHeightChange(value);
                    }
                  }}
                  onChange={(e) => {
                    const value = parseInt(e.currentTarget.value, 10);
                    if (Number.isNaN(value)) {
                      return;
                    }
                    setLocalRowHeight(value);
                    if (!isEditValue(e)) {
                      return;
                    }
                    properties.save.onRowHeightChange(value);
                  }}
                />
                <StyledHelperText>
                  {t(
                    "worksheet_settings.defaults.row_height.row_height_helper",
                  )}
                </StyledHelperText>
              </FormControl>

              <StyledLabel>
                {t("worksheet_settings.defaults.font_size.font_size")}
              </StyledLabel>
              <FormControl fullWidth>
                <StyledTextField
                  type="number"
                  value={localFontSize}
                  onBlur={(e) => {
                    const value = parseInt(e.currentTarget.value, 10);
                    if (!Number.isNaN(value)) {
                      properties.save.onFontSizeChange(value);
                    }
                  }}
                  onChange={(e) => {
                    const value = parseInt(e.currentTarget.value, 10);
                    if (Number.isNaN(value)) {
                      return;
                    }
                    setLocalFontSize(value);
                    if (!isEditValue(e)) {
                      return;
                    }
                    properties.save.onFontSizeChange(value);
                  }}
                />
                <StyledHelperText>
                  {t("worksheet_settings.defaults.font_size.font_size_helper")}
                </StyledHelperText>
              </FormControl>

              <StyledLabel>
                {t("worksheet_settings.defaults.text_color.text_color")}
              </StyledLabel>
              <FormControl fullWidth>
                <StyledColorInput
                  onClick={() => setTextColorPickerOpen(true)}
                  ref={textColorButton}
                  style={{ background: localTextColor }}
                />
                <StyledHelperText>
                  {t(
                    "worksheet_settings.defaults.text_color.text_color_helper",
                  )}
                </StyledHelperText>
              </FormControl>

              <StyledLabel>
                {t(
                  "worksheet_settings.defaults.background_color.background_color",
                )}
              </StyledLabel>
              <FormControl fullWidth>
                <StyledColorInput
                  onClick={() => setBackgroundColorPickerOpen(true)}
                  ref={backgroundColorButton}
                  style={{ background: localBackgroundColor }}
                />
                <StyledHelperText>
                  {t(
                    "worksheet_settings.defaults.background_color.background_color_helper",
                  )}
                </StyledHelperText>
              </FormControl>
            </>
          )}
        </FormSection>
      </Content>
      <ColorPicker
        color={localBackgroundColor}
        defaultColor=""
        title={t("color_picker.default")}
        onChange={(color): void => {
          setBackgroundColorPickerOpen(false);
          properties.save.onBackgroundColorChange(color);
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
        color={localTextColor}
        defaultColor="#000000"
        title={t("color_picker.default")}
        onChange={(color): void => {
          setTextColorPickerOpen(false);
          properties.save.onTextColorChange(color);
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

const ToggleBox = styled("div")`
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px;
  border: 1px solid ${theme.palette.grey[300]};
  background-color: ${theme.palette.grey[100]};
  border-radius: 6px;
`;

const ToggleText = styled("div")`
  font-size: 12px;
  font-family: Inter;
  font-weight: 500;
  color: ${theme.palette.text.primary};
`;

export default WorksheetSettingsDrawer;
