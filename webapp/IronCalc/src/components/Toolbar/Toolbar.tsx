import type {} from "@emotion/styled";
import type {
  BorderOptions,
  HorizontalAlignment,
  VerticalAlignment,
} from "@ironcalc/wasm";
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
        <ScrollArrow
          $direction="left"
          onClick={scrollLeft}
          title={t("toolbar.scroll_left")}
        >
          <ChevronLeft />
        </ScrollArrow>
      )}
      <ToolbarContainer ref={toolbarRef}>
        {/* History/Edit Group */}
        <ButtonGroup>
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
        </ButtonGroup>

        <Divider />

        {/* Format Tools Group */}
        <ButtonGroup>
          <StyledButton
            type="button"
            $pressed={false}
            onClick={properties.onCopyStyles}
            title={t("toolbar.copy_styles")}
          >
            <PaintRoller />
          </StyledButton>
          <StyledButton
            type="button"
            $pressed={false}
            onClick={() => {
              properties.onClearFormatting();
            }}
            disabled={!canEdit}
            title={t("toolbar.clear_formatting")}
          >
            <RemoveFormatting />
          </StyledButton>
        </ButtonGroup>

        <Divider />

        {/* Number Format Group */}
        <ButtonGroup>
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
            <DecimalsArrowLeft />
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
            <DecimalsArrowRight />
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
        </ButtonGroup>

        <Divider />

        {/* Font Size Group */}
        <ButtonGroup>
          <StyledButton
            type="button"
            $pressed={false}
            disabled={!canEdit}
            onClick={() => {
              properties.onIncreaseFontSize(-1);
            }}
            title={t("toolbar.decrease_font_size")}
          >
            <Minus />
          </StyledButton>
          <FontSizeBox>{properties.fontSize}</FontSizeBox>
          <StyledButton
            type="button"
            $pressed={false}
            disabled={!canEdit}
            onClick={() => {
              properties.onIncreaseFontSize(1);
            }}
            title={t("toolbar.increase_font_size")}
          >
            <Plus />
          </StyledButton>
        </ButtonGroup>

        <Divider />

        {/* Text Style Group */}
        <ButtonGroup>
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
        </ButtonGroup>

        <Divider />

        {/* Color & Border Group */}
        <ButtonGroup>
          <StyledButton
            type="button"
            $pressed={false}
            disabled={!canEdit}
            title={t("toolbar.font_color")}
            ref={fontColorButton}
            onClick={() => setFontColorPickerOpen(true)}
          >
            <Type />
            <ColorLine color={properties.fontColor} />
          </StyledButton>
          <StyledButton
            type="button"
            $pressed={false}
            disabled={!canEdit}
            title={t("toolbar.fill_color")}
            ref={fillColorButton}
            onClick={() => setFillColorPickerOpen(true)}
          >
            <PaintBucket />
            <ColorLine color={properties.fillColor} />
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
        </ButtonGroup>

        <Divider />

        {/* Alignment Group */}
        <ButtonGroup>
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
          <StyledButton
            type="button"
            $pressed={properties.wrapText === true}
            onClick={() => {
              properties.onToggleWrapText(!properties.wrapText);
            }}
            disabled={!canEdit}
            title={t("toolbar.wrap_text")}
          >
            <WrapText />
          </StyledButton>
        </ButtonGroup>

        <Divider />

        {/* View & Tools Group */}
        <ButtonGroup>
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
          <StyledButton
            type="button"
            $pressed={false}
            onClick={() => {
              properties.onDownloadPNG();
            }}
            disabled={!canEdit}
            title={t("toolbar.selected_png")}
          >
            <ImageDown />
          </StyledButton>
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
        <ScrollArrow
          $direction="right"
          onClick={scrollRight}
          title={t("toolbar.scroll_right")}
        >
          <ChevronRight />
        </ScrollArrow>
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
