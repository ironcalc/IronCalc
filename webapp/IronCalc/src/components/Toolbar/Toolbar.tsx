import type {
  BorderOptions,
  FmtSettings,
  HorizontalAlignment,
  VerticalAlignment,
} from "@ironcalc/wasm";
import { styled } from "@mui/material/styles";
import Tooltip from "@mui/material/Tooltip";
import {
  AlignCenter,
  AlignLeft,
  AlignRight,
  ArrowDownToLine,
  ArrowUpToLine,
  Bold,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  DecimalsArrowLeft,
  DecimalsArrowRight,
  DollarSign,
  Euro,
  Grid2X2,
  Grid2x2Check,
  Grid2x2X,
  ImageDown,
  Italic,
  Minus,
  PaintBucket,
  PaintRoller,
  Percent,
  Plus,
  PoundSterling,
  Redo2,
  RemoveFormatting,
  Strikethrough,
  Type,
  Underline,
  Undo2,
  WrapText,
} from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { ArrowMiddleFromLine } from "../../icons";
import BorderPicker from "../BorderPicker/BorderPicker";
import { Button } from "../Button/Button";
import { IconButton } from "../Button/IconButton";
import ColorPicker from "../ColorPicker/ColorPicker";
import { TOOLBAR_HEIGHT } from "../constants";
import FormatMenu from "../FormatMenu/FormatMenu";
import {
  decreaseDecimalPlaces,
  increaseDecimalPlaces,
  NumberFormats,
} from "../FormatMenu/formatUtil";

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
  onToggleWrapText: (v: boolean) => void;
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
  fontSize: number;
  bold: boolean;
  underline: boolean;
  italic: boolean;
  strike: boolean;
  horizontalAlign: HorizontalAlignment;
  verticalAlign: VerticalAlignment;
  wrapText: boolean;
  canEdit: boolean;
  numFmt: string;
  showGridLines: boolean;
  onToggleShowGridLines: (show: boolean) => void;
  formatOptions: FmtSettings;
};

function Toolbar(properties: ToolbarProperties) {
  const [fontColorPickerOpen, setFontColorPickerOpen] = useState(false);
  const [fillColorPickerOpen, setFillColorPickerOpen] = useState(false);
  const [borderPickerOpen, setBorderPickerOpen] = useState(false);
  const [showLeftArrow, setShowLeftArrow] = useState(false);
  const [showRightArrow, setShowRightArrow] = useState(false);

  const fontColorButton = useRef(null);
  const fillColorButton = useRef(null);
  const borderButton = useRef(null);
  const toolbarRef = useRef<HTMLDivElement>(null);

  const { t } = useTranslation();

  const { canEdit } = properties;

  const scrollLeft = () =>
    toolbarRef.current?.scrollBy({ left: -200, behavior: "smooth" });
  const scrollRight = () =>
    toolbarRef.current?.scrollBy({ left: 200, behavior: "smooth" });

  const updateArrows = useCallback(() => {
    if (!toolbarRef.current) return;
    const { scrollLeft, scrollWidth, clientWidth } = toolbarRef.current;
    setShowLeftArrow(scrollLeft > 0);
    setShowRightArrow(scrollLeft < scrollWidth - clientWidth);
  }, []);

  useEffect(() => {
    const toolbar = toolbarRef.current;
    if (!toolbar) return;

    updateArrows();
    toolbar.addEventListener("scroll", updateArrows);
    return () => toolbar.removeEventListener("scroll", updateArrows);
  }, [updateArrows]);

  let currencyIcon: React.ReactNode;

  switch (properties.formatOptions.currency) {
    case "EUR":
      currencyIcon = <Euro />;
      break;
    case "USD":
      currencyIcon = <DollarSign />;
      break;
    case "GBP":
      currencyIcon = <PoundSterling />;
      break;
  }

  return (
    <ToolbarWrapper>
      {showLeftArrow && (
        <Tooltip
          title={t("toolbar.scroll_left")}
          slotProps={{
            popper: {
              modifiers: [
                {
                  name: "offset",
                  options: {
                    offset: [0, -8],
                  },
                },
              ],
            },
          }}
        >
          <ScrollArrow $direction="left" onClick={scrollLeft}>
            <ChevronLeft />
          </ScrollArrow>
        </Tooltip>
      )}
      <ToolbarContainer ref={toolbarRef}>
        {/* History/Edit Group */}
        <ButtonGroup>
          <Tooltip title={t("toolbar.undo")}>
            <IconButton
              icon={<Undo2 />}
              aria-label="Undo"
              onClick={properties.onUndo}
              disabled={!properties.canUndo}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.redo")}>
            <IconButton
              icon={<Redo2 />}
              aria-label="Redo"
              onClick={properties.onRedo}
              disabled={!properties.canRedo}
            />
          </Tooltip>
        </ButtonGroup>

        <Divider />

        {/* Format Tools Group */}
        <ButtonGroup>
          <Tooltip title={t("toolbar.copy_styles")}>
            <IconButton
              icon={<PaintRoller />}
              aria-label="Copy Styles"
              onClick={properties.onCopyStyles}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.clear_formatting")}>
            <IconButton
              icon={<RemoveFormatting />}
              aria-label="Clear Formatting"
              onClick={() => {
                properties.onClearFormatting();
              }}
              disabled={!canEdit}
            />
          </Tooltip>
        </ButtonGroup>

        <Divider />

        {/* Number Format Group */}
        <ButtonGroup>
          <Tooltip title={t("toolbar.currency")}>
            <IconButton
              icon={currencyIcon}
              aria-label="Currency"
              onClick={(): void => {
                properties.onNumberFormatPicked(
                  properties.formatOptions.currency_format,
                );
              }}
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.percentage")}>
            <IconButton
              icon={<Percent />}
              aria-label="Percentage"
              onClick={(): void => {
                properties.onNumberFormatPicked(NumberFormats.PERCENTAGE);
              }}
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.decimal_places_decrease")}>
            <IconButton
              icon={<DecimalsArrowLeft />}
              aria-label="Decrease Decimal Places"
              onClick={(): void => {
                properties.onNumberFormatPicked(
                  decreaseDecimalPlaces(properties.numFmt),
                );
              }}
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.decimal_places_increase")}>
            <IconButton
              icon={<DecimalsArrowRight />}
              aria-label="Increase Decimal Places"
              onClick={(): void => {
                properties.onNumberFormatPicked(
                  increaseDecimalPlaces(properties.numFmt),
                );
              }}
              disabled={!canEdit}
            />
          </Tooltip>
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
            formatOptions={properties.formatOptions}
          >
            <Tooltip title={t("toolbar.format_number")}>
              <Button
                variant="ghost"
                size="sm"
                disabled={!canEdit}
                style={{ gap: 0, paddingLeft: 4, paddingRight: 2 }}
                endIcon={<ChevronDown size={12} />}
              >
                {"123"}
              </Button>
            </Tooltip>
          </FormatMenu>
        </ButtonGroup>

        <Divider />

        {/* Font Size Group */}
        <ButtonGroup>
          <Tooltip title={t("toolbar.decrease_font_size")}>
            <IconButton
              icon={<Minus />}
              aria-label="Decrease Font Size"
              onClick={() => {
                properties.onIncreaseFontSize(-1);
              }}
              disabled={!canEdit}
            />
          </Tooltip>
          <FontSizeBox>{properties.fontSize}</FontSizeBox>
          <Tooltip title={t("toolbar.increase_font_size")}>
            <IconButton
              icon={<Plus />}
              aria-label="Increase Font Size"
              onClick={() => {
                properties.onIncreaseFontSize(1);
              }}
              disabled={!canEdit}
            />
          </Tooltip>
        </ButtonGroup>

        <Divider />

        {/* Text Style Group */}
        <ButtonGroup>
          <Tooltip title={t("toolbar.bold")}>
            <IconButton
              icon={<Bold />}
              aria-label="Bold"
              pressed={properties.bold}
              onClick={() => properties.onToggleBold(!properties.bold)}
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.italic")}>
            <IconButton
              icon={<Italic />}
              aria-label="Italic"
              pressed={properties.italic}
              onClick={() => properties.onToggleItalic(!properties.italic)}
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.underline")}>
            <IconButton
              icon={<Underline />}
              aria-label="Underline"
              pressed={properties.underline}
              onClick={() =>
                properties.onToggleUnderline(!properties.underline)
              }
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.strike_through")}>
            <IconButton
              icon={<Strikethrough />}
              aria-label="Strike Through"
              pressed={properties.strike}
              onClick={() => properties.onToggleStrike(!properties.strike)}
              disabled={!canEdit}
            />
          </Tooltip>
        </ButtonGroup>

        <Divider />

        {/* Color & Border Group */}
        <ButtonGroup>
          <Tooltip title={t("toolbar.font_color")}>
            <IconButton
              type="button"
              pressed={false}
              disabled={!canEdit}
              ref={fontColorButton}
              aria-label={t("toolbar.font_color")}
              onClick={() => setFontColorPickerOpen(true)}
              icon={
                <>
                  <Type />
                  <ColorLine color={properties.fontColor} />
                </>
              }
            />
          </Tooltip>
          <Tooltip title={t("toolbar.fill_color")}>
            <IconButton
              type="button"
              pressed={false}
              disabled={!canEdit}
              ref={fillColorButton}
              aria-label={t("toolbar.fill_color")}
              onClick={() => setFillColorPickerOpen(true)}
              icon={
                <>
                  <PaintBucket />
                  <ColorLine color={properties.fillColor} />
                </>
              }
            />
          </Tooltip>
          <Tooltip title={t("toolbar.borders.title")}>
            <IconButton
              type="button"
              pressed={borderPickerOpen}
              ref={borderButton}
              aria-label={t("toolbar.borders.title")}
              onClick={() => setBorderPickerOpen(true)}
              disabled={!canEdit}
              icon={<Grid2X2 />}
            />
          </Tooltip>
        </ButtonGroup>

        <Divider />

        {/* Alignment Group */}
        <ButtonGroup>
          <Tooltip title={t("toolbar.align_left")}>
            <IconButton
              icon={<AlignLeft />}
              aria-label="Align Left"
              pressed={properties.horizontalAlign === "left"}
              onClick={() =>
                properties.onToggleHorizontalAlign(
                  properties.horizontalAlign === "left" ? "general" : "left",
                )
              }
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.align_center")}>
            <IconButton
              icon={<AlignCenter />}
              aria-label="Align Center"
              pressed={properties.horizontalAlign === "center"}
              onClick={() =>
                properties.onToggleHorizontalAlign(
                  properties.horizontalAlign === "center"
                    ? "general"
                    : "center",
                )
              }
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.align_right")}>
            <IconButton
              icon={<AlignRight />}
              aria-label="Align Right"
              pressed={properties.horizontalAlign === "right"}
              onClick={() =>
                properties.onToggleHorizontalAlign(
                  properties.horizontalAlign === "right" ? "general" : "right",
                )
              }
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.vertical_align_top")}>
            <IconButton
              icon={<ArrowUpToLine />}
              aria-label="Align Top"
              pressed={properties.verticalAlign === "top"}
              onClick={() => properties.onToggleVerticalAlign("top")}
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.vertical_align_middle")}>
            <IconButton
              icon={<ArrowMiddleFromLine />}
              aria-label="Align Middle"
              pressed={properties.verticalAlign === "center"}
              onClick={() => properties.onToggleVerticalAlign("center")}
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.vertical_align_bottom")}>
            <IconButton
              icon={<ArrowDownToLine />}
              aria-label="Align Bottom"
              pressed={properties.verticalAlign === "bottom"}
              onClick={() => properties.onToggleVerticalAlign("bottom")}
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.wrap_text")}>
            <IconButton
              icon={<WrapText />}
              aria-label="Wrap Text"
              pressed={properties.wrapText}
              onClick={() => properties.onToggleWrapText(!properties.wrapText)}
              disabled={!canEdit}
            />
          </Tooltip>
        </ButtonGroup>

        <Divider />

        {/* View & Tools Group */}
        <ButtonGroup>
          <Tooltip title={t("toolbar.show_hide_grid_lines")}>
            <IconButton
              icon={properties.showGridLines ? <Grid2x2Check /> : <Grid2x2X />}
              aria-label="Show/Hide Grid Lines"
              onClick={() =>
                properties.onToggleShowGridLines(!properties.showGridLines)
              }
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.selected_png")}>
            <IconButton
              icon={<ImageDown />}
              aria-label="Download PNG"
              onClick={() => properties.onDownloadPNG()}
              disabled={!canEdit}
            />
          </Tooltip>
        </ButtonGroup>

        <ColorPicker
          color={properties.fontColor}
          defaultColor="#000000"
          title={t("color_picker.default")}
          onChange={(color): void => {
            properties.onTextColorPicked(color);
            setFontColorPickerOpen(false);
          }}
          onClose={() => {
            setFontColorPickerOpen(false);
          }}
          anchorEl={fontColorButton}
          open={fontColorPickerOpen}
          anchorOrigin={{ vertical: "bottom", horizontal: "left" }}
          transformOrigin={{ vertical: "top", horizontal: "left" }}
        />
        <ColorPicker
          color={properties.fillColor}
          defaultColor=""
          title={t("color_picker.default")}
          onChange={(color): void => {
            if (color !== null) {
              properties.onFillColorPicked(color);
            }
            setFillColorPickerOpen(false);
          }}
          onClose={() => {
            setFillColorPickerOpen(false);
          }}
          anchorEl={fillColorButton}
          open={fillColorPickerOpen}
          anchorOrigin={{ vertical: "bottom", horizontal: "left" }}
          transformOrigin={{ vertical: "top", horizontal: "left" }}
        />
        <BorderPicker
          placement="bottom-start"
          onChange={(border): void => {
            properties.onBorderChanged(border);
          }}
          onClose={() => {
            setBorderPickerOpen(false);
          }}
          anchorEl={borderButton}
          open={borderPickerOpen}
        />
      </ToolbarContainer>
      {showRightArrow && (
        <Tooltip
          title={t("toolbar.scroll_right")}
          slotProps={{
            popper: {
              modifiers: [
                {
                  name: "offset",
                  options: {
                    offset: [0, -8],
                  },
                },
              ],
            },
          }}
        >
          <ScrollArrow $direction="right" onClick={scrollRight}>
            <ChevronRight />
          </ScrollArrow>
        </Tooltip>
      )}
    </ToolbarWrapper>
  );
}
const ToolbarWrapper = styled("div")(({ theme }) => ({
  position: "relative",
  display: "flex",
  alignItems: "center",
  background: theme.palette.background.paper,
  height: TOOLBAR_HEIGHT,
  borderBottom: `1px solid ${theme.palette.grey[300]}`,
  borderRadius: "4px 4px 0px 0px",
}));

const ToolbarContainer = styled("div")({
  display: "flex",
  flex: 1,
  alignItems: "center",
  overflowX: "auto",
  padding: "0px 8px",
  gap: 4,
  scrollbarWidth: "none",
  "&::-webkit-scrollbar": {
    display: "none",
  },
});

type TypeButtonProperties = { $pressed: boolean };
export const StyledButton = styled("button", {
  shouldForwardProp: (prop) => prop !== "$pressed",
})<TypeButtonProperties>(({ theme, disabled, $pressed }) => {
  const result = {
    width: 24,
    minWidth: 24,
    height: 24,
    display: "inline-flex",
    alignItems: "center",
    justifyContent: "center",
    fontSize: 12,
    border: `0px solid ${theme.palette.common.white}`,
    borderRadius: 4,
    transition: "all 0.2s",
    outline: `1px solid ${theme.palette.common.white}`,
    cursor: "pointer",
    backgroundColor: theme.palette.common.white,
    padding: 0,
    position: "relative" as const,
    "& svg": {
      width: 16,
      height: 16,
    },
  };

  if (disabled) {
    return {
      ...result,
      color: theme.palette.grey[400],
      cursor: "default",
    };
  }

  return {
    ...result,
    color: theme.palette.grey[900],
    backgroundColor: $pressed
      ? theme.palette.grey[300]
      : theme.palette.common.white,
    "&:hover": {
      transition: "all 0.2s",
      outline: `1px solid ${theme.palette.grey[200]}`,
    },
    "&:active": {
      backgroundColor: theme.palette.grey[300],
      outline: `1px solid ${theme.palette.grey[300]}`,
    },
  };
});

const ColorLine = styled("div")<{ color: string }>(({ color }) => ({
  height: 3,
  width: 16,
  position: "absolute",
  bottom: 0,
  left: "50%",
  transform: "translateX(-50%)",
  backgroundColor: color,
}));

const Divider = styled("div")(({ theme }) => ({
  minWidth: 1,
  height: 16,
  backgroundColor: theme.palette.grey[300],
  margin: "0px 8px",
}));

const FontSizeBox = styled("div")(({ theme }) => ({
  width: 28,
  height: 28,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  textAlign: "center",
  fontFamily: theme.typography.fontFamily,
  fontSize: 12,
  border: "none",
  borderRadius: 4,
  minWidth: 24,
}));

const ButtonGroup = styled("div")({
  display: "flex",
  alignItems: "center",
  gap: 2,
});

type ScrollArrowProps = { $direction: "left" | "right" };
const ScrollArrow = styled("button", {
  shouldForwardProp: (prop) => prop !== "$direction",
})<ScrollArrowProps>(({ theme, $direction }) => ({
  position: "absolute",
  top: "50%",
  transform: "translateY(-50%)",
  [$direction]: 0,
  zIndex: 10,
  width: 24,
  height: "100%",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  backgroundColor: theme.palette.common.white,
  border: "none",
  borderRight:
    $direction === "left" ? `1px solid ${theme.palette.grey[300]}` : "none",
  borderLeft:
    $direction === "right" ? `1px solid ${theme.palette.grey[300]}` : "none",
  cursor: "pointer",
  "&:hover": {
    backgroundColor: theme.palette.grey[100],
  },
  "& svg": {
    width: 16,
    height: 16,
  },
}));

export default Toolbar;
