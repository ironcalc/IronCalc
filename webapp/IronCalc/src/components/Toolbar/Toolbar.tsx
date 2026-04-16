import type {
  BorderOptions,
  FmtSettings,
  HorizontalAlignment,
  VerticalAlignment,
} from "@ironcalc/wasm";
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
import FormatMenu from "../FormatMenu/FormatMenu";
import {
  decreaseDecimalPlaces,
  increaseDecimalPlaces,
  NumberFormats,
} from "../FormatMenu/formatUtil";
import "./toolbar.css";
import { Tooltip } from "../Tooltip/Tooltip";

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
    <div className="ic-toolbar-wrapper">
      {showLeftArrow && (
        <Tooltip title={t("toolbar.scroll_left")}>
          {/** biome-ignore lint/a11y/noStaticElementInteractions: we need this */}
          {/** biome-ignore lint/a11y/useKeyWithClickEvents: TODO! */}
          <div
            className="ic-toolbar-scroll-arrow ic-toolbar-scroll-arrow--left"
            onClick={scrollLeft}
          >
            <ChevronLeft />
          </div>
        </Tooltip>
      )}
      <div className="ic-toolbar-container" ref={toolbarRef}>
        {/* History/Edit Group */}
        <div className="ic-toolbar-button-group">
          <Tooltip title={t("toolbar.undo")}>
            <IconButton
              icon={<Undo2 />}
              aria-label={t("toolbar.undo")}
              onClick={properties.onUndo}
              disabled={!properties.canUndo}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.redo")}>
            <IconButton
              icon={<Redo2 />}
              aria-label={t("toolbar.redo")}
              onClick={properties.onRedo}
              disabled={!properties.canRedo}
            />
          </Tooltip>
        </div>

        <div className="ic-toolbar-divider" />

        {/* Format Tools Group */}
        <div className="ic-toolbar-button-group">
          <Tooltip title={t("toolbar.copy_styles")}>
            <IconButton
              icon={<PaintRoller />}
              aria-label={t("toolbar.copy_styles")}
              onClick={properties.onCopyStyles}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.clear_formatting")}>
            <IconButton
              icon={<RemoveFormatting />}
              aria-label={t("toolbar.clear_formatting")}
              onClick={() => {
                properties.onClearFormatting();
              }}
              disabled={!canEdit}
            />
          </Tooltip>
        </div>

        <div className="ic-toolbar-divider" />

        {/* Number Format Group */}
        <div className="ic-toolbar-button-group">
          <Tooltip title={t("toolbar.currency")}>
            <IconButton
              icon={currencyIcon}
              aria-label={t("toolbar.currency")}
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
              aria-label={t("toolbar.percentage")}
              onClick={(): void => {
                properties.onNumberFormatPicked(NumberFormats.PERCENTAGE);
              }}
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.decimal_places_decrease")}>
            <IconButton
              icon={<DecimalsArrowLeft />}
              aria-label={t("toolbar.decimal_places_decrease")}
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
              aria-label={t("toolbar.decimal_places_increase")}
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
        </div>

        <div className="ic-toolbar-divider" />

        {/* Font Size Group */}
        <div className="ic-toolbar-button-group">
          <Tooltip title={t("toolbar.decrease_font_size")}>
            <IconButton
              icon={<Minus />}
              aria-label={t("toolbar.decrease_font_size")}
              onClick={() => {
                properties.onIncreaseFontSize(-1);
              }}
              disabled={!canEdit}
            />
          </Tooltip>
          <div className="ic-toolbar-font-size-box">{properties.fontSize}</div>
          <Tooltip title={t("toolbar.increase_font_size")}>
            <IconButton
              icon={<Plus />}
              aria-label={t("toolbar.increase_font_size")}
              onClick={() => {
                properties.onIncreaseFontSize(1);
              }}
              disabled={!canEdit}
            />
          </Tooltip>
        </div>

        <div className="ic-toolbar-divider" />

        {/* Text Style Group */}
        <div className="ic-toolbar-button-group">
          <Tooltip title={t("toolbar.bold")}>
            <IconButton
              icon={<Bold />}
              aria-label={t("toolbar.bold")}
              pressed={properties.bold}
              onClick={() => properties.onToggleBold(!properties.bold)}
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.italic")}>
            <IconButton
              icon={<Italic />}
              aria-label={t("toolbar.italic")}
              pressed={properties.italic}
              onClick={() => properties.onToggleItalic(!properties.italic)}
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.underline")}>
            <IconButton
              icon={<Underline />}
              aria-label={t("toolbar.underline")}
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
              aria-label={t("toolbar.strike_through")}
              pressed={properties.strike}
              onClick={() => properties.onToggleStrike(!properties.strike)}
              disabled={!canEdit}
            />
          </Tooltip>
        </div>

        <div className="ic-toolbar-divider" />

        {/* Color & Border Group */}
        <div className="ic-toolbar-button-group">
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
                  <div
                    className="ic-toolbar-color-line"
                    style={{ backgroundColor: properties.fontColor }}
                  />
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
                  <div
                    className="ic-toolbar-color-line"
                    style={{ backgroundColor: properties.fillColor }}
                  />
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
        </div>

        <div className="ic-toolbar-divider" />

        {/* Alignment Group */}
        <div className="ic-toolbar-button-group">
          <Tooltip title={t("toolbar.align_left")}>
            <IconButton
              icon={<AlignLeft />}
              aria-label={t("toolbar.align_left")}
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
              aria-label={t("toolbar.align_center")}
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
              aria-label={t("toolbar.align_right")}
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
              aria-label={t("toolbar.vertical_align_top")}
              pressed={properties.verticalAlign === "top"}
              onClick={() => properties.onToggleVerticalAlign("top")}
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.vertical_align_middle")}>
            <IconButton
              icon={<ArrowMiddleFromLine />}
              aria-label={t("toolbar.vertical_align_middle")}
              pressed={properties.verticalAlign === "center"}
              onClick={() => properties.onToggleVerticalAlign("center")}
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.vertical_align_bottom")}>
            <IconButton
              icon={<ArrowDownToLine />}
              aria-label={t("toolbar.vertical_align_bottom")}
              pressed={properties.verticalAlign === "bottom"}
              onClick={() => properties.onToggleVerticalAlign("bottom")}
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.wrap_text")}>
            <IconButton
              icon={<WrapText />}
              aria-label={t("toolbar.wrap_text")}
              pressed={properties.wrapText}
              onClick={() => properties.onToggleWrapText(!properties.wrapText)}
              disabled={!canEdit}
            />
          </Tooltip>
        </div>

        <div className="ic-toolbar-divider" />

        {/* View & Tools Group */}
        <div className="ic-toolbar-button-group">
          <Tooltip title={t("toolbar.show_hide_grid_lines")}>
            <IconButton
              icon={properties.showGridLines ? <Grid2x2Check /> : <Grid2x2X />}
              aria-label={t("toolbar.show_hide_grid_lines")}
              onClick={() =>
                properties.onToggleShowGridLines(!properties.showGridLines)
              }
              disabled={!canEdit}
            />
          </Tooltip>
          <Tooltip title={t("toolbar.selected_png")}>
            <IconButton
              icon={<ImageDown />}
              aria-label={t("toolbar.download_png")}
              onClick={() => properties.onDownloadPNG()}
              disabled={!canEdit}
            />
          </Tooltip>
        </div>

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
      </div>
      {showRightArrow && (
        <Tooltip title={t("toolbar.scroll_right")}>
          {/** biome-ignore lint/a11y/noStaticElementInteractions: we need this */}
          {/** biome-ignore lint/a11y/useKeyWithClickEvents: TODO! */}
          <div
            className="ic-toolbar-scroll-arrow ic-toolbar-scroll-arrow--right"
            onClick={scrollRight}
          >
            <ChevronRight />
          </div>
        </Tooltip>
      )}
    </div>
  );
}

export default Toolbar;
