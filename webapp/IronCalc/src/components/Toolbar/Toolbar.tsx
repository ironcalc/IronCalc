import type {} from "@emotion/styled";
import type {
  BorderOptions,
  HorizontalAlignment,
  VerticalAlignment,
} from "@ironcalc/wasm";
import Tooltip from "@mui/material/Tooltip";
import { styled } from "@mui/material/styles";
import type {} from "@mui/system";
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
  Redo2,
  RemoveFormatting,
  Strikethrough,
  Tags,
  Type,
  Underline,
  Undo2,
  WrapText,
} from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { ArrowMiddleFromLine } from "../../icons";
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
  nameManagerProperties: NameManagerProperties;
};

function Toolbar(properties: ToolbarProperties) {
  const [fontColorPickerOpen, setFontColorPickerOpen] = useState(false);
  const [fillColorPickerOpen, setFillColorPickerOpen] = useState(false);
  const [borderPickerOpen, setBorderPickerOpen] = useState(false);
  const [nameManagerDialogOpen, setNameManagerDialogOpen] = useState(false);
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
            <StyledButton
              type="button"
              $pressed={false}
              onClick={properties.onUndo}
              disabled={!properties.canUndo}
            >
              <Undo2 />
            </StyledButton>
          </Tooltip>
          <Tooltip title={t("toolbar.redo")}>
            <StyledButton
              type="button"
              $pressed={false}
              onClick={properties.onRedo}
              disabled={!properties.canRedo}
            >
              <Redo2 />
            </StyledButton>
          </Tooltip>
        </ButtonGroup>

        <Divider />

        {/* Format Tools Group */}
        <ButtonGroup>
          <Tooltip title={t("toolbar.copy_styles")}>
            <StyledButton
              type="button"
              $pressed={false}
              onClick={properties.onCopyStyles}
            >
              <PaintRoller />
            </StyledButton>
          </Tooltip>
          <Tooltip title={t("toolbar.clear_formatting")}>
            <StyledButton
              type="button"
              $pressed={false}
              onClick={() => {
                properties.onClearFormatting();
              }}
              disabled={!canEdit}
            >
              <RemoveFormatting />
            </StyledButton>
          </Tooltip>
        </ButtonGroup>

        <Divider />

        {/* Number Format Group */}
        <ButtonGroup>
          <Tooltip title={t("toolbar.euro")}>
            <StyledButton
              type="button"
              $pressed={false}
              onClick={(): void => {
                properties.onNumberFormatPicked(NumberFormats.CURRENCY_EUR);
              }}
              disabled={!canEdit}
            >
              <Euro />
            </StyledButton>
          </Tooltip>
          <Tooltip title={t("toolbar.percentage")}>
            <StyledButton
              type="button"
              $pressed={false}
              onClick={(): void => {
                properties.onNumberFormatPicked(NumberFormats.PERCENTAGE);
              }}
              disabled={!canEdit}
            >
              <Percent />
            </StyledButton>
          </Tooltip>
          <Tooltip title={t("toolbar.decimal_places_decrease")}>
            <StyledButton
              type="button"
              $pressed={false}
              onClick={(): void => {
                properties.onNumberFormatPicked(
                  decreaseDecimalPlaces(properties.numFmt),
                );
              }}
              disabled={!canEdit}
            >
              <DecimalsArrowLeft />
            </StyledButton>
          </Tooltip>
          <Tooltip title={t("toolbar.decimal_places_increase")}>
            <StyledButton
              type="button"
              $pressed={false}
              onClick={(): void => {
                properties.onNumberFormatPicked(
                  increaseDecimalPlaces(properties.numFmt),
                );
              }}
              disabled={!canEdit}
            >
              <DecimalsArrowRight />
            </StyledButton>
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
          >
            <Tooltip title={t("toolbar.format_number")}>
              <StyledButton
                type="button"
                $pressed={false}
                disabled={!canEdit}
                sx={{
                  width: "40px", // Keep in sync with anchorOrigin in FormatMenu above
                  padding: "0px 4px",
                }}
              >
                {"123"}
                <ChevronDown />
              </StyledButton>
            </Tooltip>
          </FormatMenu>
        </ButtonGroup>

        <Divider />

        {/* Font Size Group */}
        <ButtonGroup>
          <Tooltip title={t("toolbar.decrease_font_size")}>
            <StyledButton
              type="button"
              $pressed={false}
              disabled={!canEdit}
              onClick={() => {
                properties.onIncreaseFontSize(-1);
              }}
            >
              <Minus />
            </StyledButton>
          </Tooltip>
          <FontSizeBox>{properties.fontSize}</FontSizeBox>
          <Tooltip title={t("toolbar.increase_font_size")}>
            <StyledButton
              type="button"
              $pressed={false}
              disabled={!canEdit}
              onClick={() => {
                properties.onIncreaseFontSize(1);
              }}
            >
              <Plus />
            </StyledButton>
          </Tooltip>
        </ButtonGroup>

        <Divider />

        {/* Text Style Group */}
        <ButtonGroup>
          <Tooltip title={t("toolbar.bold")}>
            <StyledButton
              type="button"
              $pressed={properties.bold}
              onClick={() => properties.onToggleBold(!properties.bold)}
              disabled={!canEdit}
            >
              <Bold />
            </StyledButton>
          </Tooltip>
          <Tooltip title={t("toolbar.italic")}>
            <StyledButton
              type="button"
              $pressed={properties.italic}
              onClick={() => properties.onToggleItalic(!properties.italic)}
              disabled={!canEdit}
            >
              <Italic />
            </StyledButton>
          </Tooltip>
          <Tooltip title={t("toolbar.underline")}>
            <StyledButton
              type="button"
              $pressed={properties.underline}
              onClick={() =>
                properties.onToggleUnderline(!properties.underline)
              }
              disabled={!canEdit}
            >
              <Underline />
            </StyledButton>
          </Tooltip>
          <Tooltip title={t("toolbar.strike_through")}>
            <StyledButton
              type="button"
              $pressed={properties.strike}
              onClick={() => properties.onToggleStrike(!properties.strike)}
              disabled={!canEdit}
            >
              <Strikethrough />
            </StyledButton>
          </Tooltip>
        </ButtonGroup>

        <Divider />

        {/* Color & Border Group */}
        <ButtonGroup>
          <Tooltip title={t("toolbar.font_color")}>
            <StyledButton
              type="button"
              $pressed={false}
              disabled={!canEdit}
              ref={fontColorButton}
              onClick={() => setFontColorPickerOpen(true)}
            >
              <Type />
              <ColorLine color={properties.fontColor} />
            </StyledButton>
          </Tooltip>
          <Tooltip title={t("toolbar.fill_color")}>
            <StyledButton
              type="button"
              $pressed={false}
              disabled={!canEdit}
              ref={fillColorButton}
              onClick={() => setFillColorPickerOpen(true)}
            >
              <PaintBucket />
              <ColorLine color={properties.fillColor} />
            </StyledButton>
          </Tooltip>
          <Tooltip title={t("toolbar.borders.title")}>
            <StyledButton
              type="button"
              $pressed={false}
              onClick={() => setBorderPickerOpen(true)}
              ref={borderButton}
              disabled={!canEdit}
            >
              <Grid2X2 />
            </StyledButton>
          </Tooltip>
        </ButtonGroup>

        <Divider />

        {/* Alignment Group */}
        <ButtonGroup>
          <Tooltip title={t("toolbar.align_left")}>
            <StyledButton
              type="button"
              $pressed={properties.horizontalAlign === "left"}
              onClick={() =>
                properties.onToggleHorizontalAlign(
                  properties.horizontalAlign === "left" ? "general" : "left",
                )
              }
              disabled={!canEdit}
            >
              <AlignLeft />
            </StyledButton>
          </Tooltip>
          <Tooltip title={t("toolbar.align_center")}>
            <StyledButton
              type="button"
              $pressed={properties.horizontalAlign === "center"}
              onClick={() =>
                properties.onToggleHorizontalAlign(
                  properties.horizontalAlign === "center"
                    ? "general"
                    : "center",
                )
              }
              disabled={!canEdit}
            >
              <AlignCenter />
            </StyledButton>
          </Tooltip>
          <Tooltip title={t("toolbar.align_right")}>
            <StyledButton
              type="button"
              $pressed={properties.horizontalAlign === "right"}
              onClick={() =>
                properties.onToggleHorizontalAlign(
                  properties.horizontalAlign === "right" ? "general" : "right",
                )
              }
              disabled={!canEdit}
            >
              <AlignRight />
            </StyledButton>
          </Tooltip>
          <Tooltip title={t("toolbar.vertical_align_top")}>
            <StyledButton
              type="button"
              $pressed={properties.verticalAlign === "top"}
              onClick={() => properties.onToggleVerticalAlign("top")}
              disabled={!canEdit}
            >
              <ArrowUpToLine />
            </StyledButton>
          </Tooltip>
          <Tooltip title={t("toolbar.vertical_align_middle")}>
            <StyledButton
              type="button"
              $pressed={properties.verticalAlign === "center"}
              onClick={() => properties.onToggleVerticalAlign("center")}
              disabled={!canEdit}
            >
              <ArrowMiddleFromLine />
            </StyledButton>
          </Tooltip>
          <Tooltip title={t("toolbar.vertical_align_bottom")}>
            <StyledButton
              type="button"
              $pressed={properties.verticalAlign === "bottom"}
              onClick={() => properties.onToggleVerticalAlign("bottom")}
              disabled={!canEdit}
            >
              <ArrowDownToLine />
            </StyledButton>
          </Tooltip>
          <Tooltip title={t("toolbar.wrap_text")}>
            <StyledButton
              type="button"
              $pressed={properties.wrapText === true}
              onClick={() => {
                properties.onToggleWrapText(!properties.wrapText);
              }}
              disabled={!canEdit}
            >
              <WrapText />
            </StyledButton>
          </Tooltip>
        </ButtonGroup>

        <Divider />

        {/* View & Tools Group */}
        <ButtonGroup>
          <Tooltip title={t("toolbar.show_hide_grid_lines")}>
            <StyledButton
              type="button"
              $pressed={false}
              onClick={() =>
                properties.onToggleShowGridLines(!properties.showGridLines)
              }
              disabled={!canEdit}
            >
              {properties.showGridLines ? <Grid2x2Check /> : <Grid2x2X />}
            </StyledButton>
          </Tooltip>
          <Tooltip title={t("toolbar.name_manager")}>
            <StyledButton
              type="button"
              $pressed={false}
              onClick={() => {
                setNameManagerDialogOpen(true);
              }}
              disabled={!canEdit}
            >
              <Tags />
            </StyledButton>
          </Tooltip>
          <Tooltip title={t("toolbar.selected_png")}>
            <StyledButton
              type="button"
              $pressed={false}
              onClick={() => {
                properties.onDownloadPNG();
              }}
              disabled={!canEdit}
            >
              <ImageDown />
            </StyledButton>
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

const ToolbarWrapper = styled("div")`
  position: relative;
  display: flex;
  align-items: center;
  background: ${({ theme }) => theme.palette.background.paper};
  height: ${TOOLBAR_HEIGHT}px;
  border-bottom: 1px solid ${({ theme }) => theme.palette.grey["300"]};
  border-radius: 4px 4px 0px 0px;
`;

const ToolbarContainer = styled("div")`
  display: flex;
  flex: 1;
  align-items: center;
  overflow-x: auto;
  padding: 0px 12px;
  gap: 4px;
  scrollbar-width: none;
  &::-webkit-scrollbar {
    display: none;
  }
`;

type TypeButtonProperties = { $pressed: boolean };
export const StyledButton = styled("button", {
  shouldForwardProp: (prop) => prop !== "$pressed",
})<TypeButtonProperties>(({ disabled, $pressed }) => {
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
    position: "relative" as const,
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
    color: theme.palette.grey["900"],
    backgroundColor: $pressed
      ? theme.palette.grey["300"]
      : theme.palette.common.white,
    "&:hover": {
      transition: "all 0.2s",
      outline: `1px solid ${theme.palette.grey["200"]}`,
    },
    "&:active": {
      backgroundColor: theme.palette.grey["300"],
      outline: `1px solid ${theme.palette.grey["300"]}`,
    },
  };
});

const ColorLine = styled("div")<{ color: string }>(({ color }) => ({
  height: "3px",
  width: "16px",
  position: "absolute",
  bottom: "0px",
  left: "50%",
  transform: "translateX(-50%)",
  backgroundColor: color,
}));

const Divider = styled("div")({
  minWidth: "1px",
  height: "16px",
  backgroundColor: theme.palette.grey["300"],
  margin: "0px 8px",
});

const FontSizeBox = styled("div")({
  width: "24px",
  height: "24px",
  lineHeight: "24px",
  textAlign: "center",
  fontFamily: "Inter",
  fontSize: "11px",
  border: `1px solid ${theme.palette.grey["300"]}`,
  borderRadius: "4px",
  minWidth: "24px",
});

const ButtonGroup = styled("div")({
  display: "flex",
  alignItems: "center",
  gap: "4px",
});

type ScrollArrowProps = { $direction: "left" | "right" };
const ScrollArrow = styled("button", {
  shouldForwardProp: (prop) => prop !== "$direction",
})<ScrollArrowProps>(({ $direction }) => ({
  position: "absolute",
  top: "50%",
  transform: "translateY(-50%)",
  [$direction]: "0px",
  zIndex: 10,
  width: "24px",
  height: "100%",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  backgroundColor: "white",
  border:
    $direction === "left"
      ? `none; border-right: 1px solid ${theme.palette.grey["300"]};`
      : `none; border-left: 1px solid ${theme.palette.grey["300"]};`,
  cursor: "pointer",
  "&:hover": {
    backgroundColor: theme.palette.grey["100"],
  },
  svg: {
    width: "16px",
    height: "16px",
  },
}));

export default Toolbar;
