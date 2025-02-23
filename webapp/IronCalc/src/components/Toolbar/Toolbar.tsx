import type {} from "@emotion/styled";
import type {
  BorderOptions,
  HorizontalAlignment,
  VerticalAlignment,
} from "@ironcalc/wasm";
import { styled } from "@mui/material/styles";
import type {} from "@mui/system";
import {
  AArrowDown,
  AArrowUp,
  AlignCenter,
  AlignLeft,
  AlignRight,
  ArrowDownToLine,
  ArrowUpToLine,
  Bold,
  ChevronDown,
  Euro,
  Grid2X2,
  Grid2x2Check,
  Grid2x2X,
  ImageDown,
  Italic,
  PaintBucket,
  PaintRoller,
  Percent,
  Redo2,
  RemoveFormatting,
  Strikethrough,
  Tags,
  Type,
  Underline,
  Undo2,
} from "lucide-react";
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  ArrowMiddleFromLine,
  DecimalPlacesDecreaseIcon,
  DecimalPlacesIncreaseIcon,
} from "../../icons";
import { theme } from "../../theme";
import BorderPicker from "../BorderPicker/BorderPicker";
import ColorPicker from "../ColorPicker/ColorPicker";
import FormatMenu from "../FormatMenu/FormatMenu";
import {
  NumberFormats,
  decreaseDecimalPlaces,
  increaseDecimalPlaces,
} from "../FormatMenu/formatUtil";
import NameManagerDialog from "../NameManagerDialog";
import type { NameManagerProperties } from "../NameManagerDialog/NameManagerDialog";
import { TOOLBAR_HEIGHT } from "../constants";

type ToolbarProperties = {
  canUndo: boolean;
  canRedo: boolean;
  onRedo: () => void;
  onUndo: () => void;
  onToggleUnderline: (u: boolean) => void;
  onToggleBold: (v: boolean) => void;
  onToggleItalic: (v: boolean) => void;
  onToggleStrike: (v: boolean) => void;
  onToggleHorizontalAlign: (v: string) => void;
  onToggleVerticalAlign: (v: string) => void;
  onCopyStyles: () => void;
  onTextColorPicked: (hex: string) => void;
  onFillColorPicked: (hex: string) => void;
  onNumberFormatPicked: (numberFmt: string) => void;
  onBorderChanged: (border: BorderOptions) => void;
  onClearFormatting: () => void;
  onIncreaseFontSize: (delta: number) => void;
  onDownloadPNG: () => void;
  fillColor: string;
  fontColor: string;
  bold: boolean;
  underline: boolean;
  italic: boolean;
  strike: boolean;
  horizontalAlign: HorizontalAlignment;
  verticalAlign: VerticalAlignment;
  canEdit: boolean;
  numFmt: string;
  showGridLines: boolean;
  onToggleShowGridLines: (show: boolean) => void;
  nameManagerProperties: NameManagerProperties;
};

function Toolbar(properties: ToolbarProperties) {
  const [fontColorPickerOpen, setFontColorPickerOpen] = useState(false);
  const [fillColorPickerOpen, setFillColorPickerOpen] = useState(false);
  const [borderPickerOpen, setBorderPickerOpen] = useState(false);
  const [nameManagerDialogOpen, setNameManagerDialogOpen] = useState(false);

  const fontColorButton = useRef(null);
  const fillColorButton = useRef(null);
  const borderButton = useRef(null);

  const { t } = useTranslation();

  const { canEdit } = properties;

  return (
    <ToolbarContainer>
      <StyledButton
        type="button"
        $pressed={false}
        onClick={properties.onUndo}
        disabled={!properties.canUndo}
        title={t("toolbar.undo")}
      >
        <Undo2 />
      </StyledButton>
      <StyledButton
        type="button"
        $pressed={false}
        onClick={properties.onRedo}
        disabled={!properties.canRedo}
        title={t("toolbar.redo")}
      >
        <Redo2 />
      </StyledButton>
      <Divider />
      <StyledButton
        type="button"
        $pressed={false}
        onClick={properties.onCopyStyles}
        title={t("toolbar.copy_styles")}
      >
        <PaintRoller />
      </StyledButton>
      <Divider />
      <StyledButton
        type="button"
        $pressed={false}
        onClick={(): void => {
          properties.onNumberFormatPicked(NumberFormats.CURRENCY_EUR);
        }}
        disabled={!canEdit}
        title={t("toolbar.euro")}
      >
        <Euro />
      </StyledButton>
      <StyledButton
        type="button"
        $pressed={false}
        onClick={(): void => {
          properties.onNumberFormatPicked(NumberFormats.PERCENTAGE);
        }}
        disabled={!canEdit}
        title={t("toolbar.percentage")}
      >
        <Percent />
      </StyledButton>
      <StyledButton
        type="button"
        $pressed={false}
        onClick={(): void => {
          properties.onNumberFormatPicked(
            decreaseDecimalPlaces(properties.numFmt),
          );
        }}
        disabled={!canEdit}
        title={t("toolbar.decimal_places_decrease")}
      >
        <DecimalPlacesDecreaseIcon />
      </StyledButton>
      <StyledButton
        type="button"
        $pressed={false}
        onClick={(): void => {
          properties.onNumberFormatPicked(
            increaseDecimalPlaces(properties.numFmt),
          );
        }}
        disabled={!canEdit}
        title={t("toolbar.decimal_places_increase")}
      >
        <DecimalPlacesIncreaseIcon />
      </StyledButton>
      <FormatMenu
        numFmt={properties.numFmt}
        onChange={(numberFmt): void => {
          properties.onNumberFormatPicked(numberFmt);
        }}
        onExited={(): void => {}}
        anchorOrigin={{
          horizontal: 20, // Aligning the menu to the middle of FormatButton
          vertical: "bottom",
        }}
      >
        <StyledButton
          type="button"
          $pressed={false}
          disabled={!canEdit}
          title={t("toolbar.format_number")}
          sx={{
            width: "40px", // Keep in sync with anchorOrigin in FormatMenu above
            padding: "0px 4px",
          }}
        >
          {"123"}
          <ChevronDown />
        </StyledButton>
      </FormatMenu>
      <Divider />
      <StyledButton
        type="button"
        $pressed={properties.bold}
        onClick={() => properties.onToggleBold(!properties.bold)}
        disabled={!canEdit}
        title={t("toolbar.bold")}
      >
        <Bold />
      </StyledButton>
      <StyledButton
        type="button"
        $pressed={properties.italic}
        onClick={() => properties.onToggleItalic(!properties.italic)}
        disabled={!canEdit}
        title={t("toolbar.italic")}
      >
        <Italic />
      </StyledButton>
      <StyledButton
        type="button"
        $pressed={properties.underline}
        onClick={() => properties.onToggleUnderline(!properties.underline)}
        disabled={!canEdit}
        title={t("toolbar.underline")}
      >
        <Underline />
      </StyledButton>
      <StyledButton
        type="button"
        $pressed={properties.strike}
        onClick={() => properties.onToggleStrike(!properties.strike)}
        disabled={!canEdit}
        title={t("toolbar.strike_through")}
      >
        <Strikethrough />
      </StyledButton>
      <Divider />
      <StyledButton
        type="button"
        $pressed={false}
        disabled={!canEdit}
        title={t("toolbar.font_color")}
        ref={fontColorButton}
        $underlinedColor={properties.fontColor}
        onClick={() => setFontColorPickerOpen(true)}
      >
        <Type />
      </StyledButton>
      <StyledButton
        type="button"
        $pressed={false}
        disabled={!canEdit}
        onClick={() => {
          properties.onIncreaseFontSize(1);
        }}
        title={t("toolbar.increase_font_size")}
      >
        <AArrowUp />
      </StyledButton>
      <StyledButton
        type="button"
        $pressed={false}
        disabled={!canEdit}
        onClick={() => {
          properties.onIncreaseFontSize(-1);
        }}
        title={t("toolbar.decrease_font_size")}
      >
        <AArrowDown />
      </StyledButton>
      <StyledButton
        type="button"
        $pressed={false}
        disabled={!canEdit}
        title={t("toolbar.fill_color")}
        ref={fillColorButton}
        $underlinedColor={properties.fillColor}
        onClick={() => setFillColorPickerOpen(true)}
      >
        <PaintBucket />
      </StyledButton>
      <StyledButton
        type="button"
        $pressed={false}
        onClick={() => setBorderPickerOpen(true)}
        ref={borderButton}
        disabled={!canEdit}
        title={t("toolbar.borders.title")}
      >
        <Grid2X2 />
      </StyledButton>
      <Divider />
      <StyledButton
        type="button"
        $pressed={properties.horizontalAlign === "left"}
        onClick={() =>
          properties.onToggleHorizontalAlign(
            properties.horizontalAlign === "left" ? "general" : "left",
          )
        }
        disabled={!canEdit}
        title={t("toolbar.align_left")}
      >
        <AlignLeft />
      </StyledButton>
      <StyledButton
        type="button"
        $pressed={properties.horizontalAlign === "center"}
        onClick={() =>
          properties.onToggleHorizontalAlign(
            properties.horizontalAlign === "center" ? "general" : "center",
          )
        }
        disabled={!canEdit}
        title={t("toolbar.align_center")}
      >
        <AlignCenter />
      </StyledButton>
      <StyledButton
        type="button"
        $pressed={properties.horizontalAlign === "right"}
        onClick={() =>
          properties.onToggleHorizontalAlign(
            properties.horizontalAlign === "right" ? "general" : "right",
          )
        }
        disabled={!canEdit}
        title={t("toolbar.align_right")}
      >
        <AlignRight />
      </StyledButton>
      <StyledButton
        type="button"
        $pressed={properties.verticalAlign === "top"}
        onClick={() => properties.onToggleVerticalAlign("top")}
        disabled={!canEdit}
        title={t("toolbar.vertical_align_top")}
      >
        <ArrowUpToLine />
      </StyledButton>
      <StyledButton
        type="button"
        $pressed={properties.verticalAlign === "center"}
        onClick={() => properties.onToggleVerticalAlign("center")}
        disabled={!canEdit}
        title={t("toolbar.vertical_align_middle")}
      >
        <ArrowMiddleFromLine />
      </StyledButton>
      <StyledButton
        type="button"
        $pressed={properties.verticalAlign === "bottom"}
        onClick={() => properties.onToggleVerticalAlign("bottom")}
        disabled={!canEdit}
        title={t("toolbar.vertical_align_bottom")}
      >
        <ArrowDownToLine />
      </StyledButton>

      <Divider />
      <StyledButton
        type="button"
        $pressed={false}
        onClick={() =>
          properties.onToggleShowGridLines(!properties.showGridLines)
        }
        disabled={!canEdit}
        title={t("toolbar.show_hide_grid_lines")}
      >
        {properties.showGridLines ? <Grid2x2Check /> : <Grid2x2X />}
      </StyledButton>
      <Divider />
      <StyledButton
        type="button"
        $pressed={false}
        onClick={() => {
          setNameManagerDialogOpen(true);
        }}
        disabled={!canEdit}
        title={t("toolbar.name_manager")}
      >
        <Tags />
      </StyledButton>

      <Divider />
      <StyledButton
        type="button"
        $pressed={false}
        disabled={!canEdit}
        onClick={() => {
          properties.onClearFormatting();
        }}
        title={t("toolbar.clear_formatting")}
      >
        <RemoveFormatting />
      </StyledButton>
      <StyledButton
        type="button"
        $pressed={false}
        disabled={!canEdit}
        onClick={() => {
          properties.onDownloadPNG();
        }}
        title={t("toolbar.selected_png")}
      >
        <ImageDown />
      </StyledButton>

      <ColorPicker
        color={properties.fontColor}
        onChange={(color): void => {
          properties.onTextColorPicked(color);
          setFontColorPickerOpen(false);
        }}
        onClose={() => {
          setFontColorPickerOpen(false);
        }}
        anchorEl={fontColorButton}
        open={fontColorPickerOpen}
      />
      <ColorPicker
        color={properties.fillColor}
        onChange={(color): void => {
          properties.onFillColorPicked(color);
          setFillColorPickerOpen(false);
        }}
        onClose={() => {
          setFillColorPickerOpen(false);
        }}
        anchorEl={fillColorButton}
        open={fillColorPickerOpen}
      />
      <BorderPicker
        onChange={(border): void => {
          properties.onBorderChanged(border);
        }}
        onClose={() => {
          setBorderPickerOpen(false);
        }}
        anchorEl={borderButton}
        open={borderPickerOpen}
      />
      <NameManagerDialog
        open={nameManagerDialogOpen}
        onClose={() => {
          setNameManagerDialogOpen(false);
        }}
        model={properties.nameManagerProperties}
      />
    </ToolbarContainer>
  );
}

const ToolbarContainer = styled("div")`
  display: flex;
  flex-shrink: 0;
  align-items: center;
  background: ${({ theme }) => theme.palette.background.paper};
  height: ${TOOLBAR_HEIGHT}px;
  line-height: ${TOOLBAR_HEIGHT}px;
  border-bottom: 1px solid ${({ theme }) => theme.palette.grey["300"]};
  font-family: Inter;
  border-radius: 4px 4px 0px 0px;
  overflow-x: auto;
  padding: 0px 12px;
  gap: 4px;
  scrollbar-width: none;
`;

type TypeButtonProperties = { $pressed: boolean; $underlinedColor?: string };
export const StyledButton = styled("button")<TypeButtonProperties>(
  ({ disabled, $pressed, $underlinedColor }) => {
    const result = {
      width: "24px",
      minWidth: "24px",
      height: "24px",
      display: "inline-flex",
      alignItems: "center",
      justifyContent: "center",
      fontSize: "12px",
      border: `0px solid ${theme.palette.common.white}`,
      borderRadius: "4px",
      transition: "all 0.2s",
      outline: `1px solid ${theme.palette.common.white}`,
      cursor: "pointer",
      backgroundColor: "white",
      padding: "0px",
      svg: {
        width: "16px",
        height: "16px",
      },
    };
    if (disabled) {
      return {
        ...result,
        color: theme.palette.grey["400"],
        cursor: "default",
      };
    }
    return {
      ...result,
      borderTop: $underlinedColor
        ? `3px solid ${theme.palette.common.white}`
        : "none",
      borderBottom: $underlinedColor ? `3px solid ${$underlinedColor}` : "none",
      color: theme.palette.grey["900"],
      backgroundColor: $pressed
        ? theme.palette.grey["300"]
        : theme.palette.common.white,
      "&:hover": {
        transition: "all 0.2s",
        outline: `1px solid ${theme.palette.grey["200"]}`,
        borderTopColor: theme.palette.common.white,
      },
      "&:active": {
        backgroundColor: theme.palette.grey["300"],
        outline: `1px solid ${theme.palette.grey["300"]}`,
      },
    };
  },
);

const Divider = styled("div")({
  width: "0px",
  height: "12px",
  borderLeft: `1px solid ${theme.palette.grey["300"]}`,
  margin: "0px 12px",
});

export default Toolbar;
