import { JSX } from "solid-js/jsx-runtime";
import styles from "./toolbar.module.css";
import {
  AlignCenter,
  AlignLeft,
  AlignRight,
  ArrowDownToLine,
  ArrowUpToLine,
  Bold,
  ChevronDown,
  Euro,
  Grid2X2,
  Italic,
  Paintbrush2,
  PaintBucket,
  Percent,
  Redo2,
  Strikethrough,
  Type,
  Underline,
  Undo2,
} from "lucide-solid";
import { DecimalPlacesDecreaseIcon, DecimalPlacesIncreaseIcon, ArrowMiddleFromLine } from "../../icons";

function Toolbar() {
  const onkeydown = (event: KeyboardEvent) => {
    console.log("key pressed: ", event);
  };

  const t = (s: string): string => s;

  const properties = {
    onUndo: () => {},
    canUndo: true,
    onRedo: () => {},
    canRedo: true,
    onCopyStyles: () => {},
    canEdit: true,
  };

  return (
    <div class={styles.toolbar} onkeydown={onkeydown} tabIndex={0}>
      <StyledButton
        pressed={false}
        onClick={properties.onUndo}
        disabled={!properties.canUndo}
        title={t("toolbar.undo")}
      >
        <Undo2 />
      </StyledButton>
      <StyledButton
        pressed={false}
        onClick={properties.onRedo}
        disabled={!properties.canRedo}
        title={t("toolbar.redo")}
      >
        <Redo2 />
      </StyledButton>
      <div class={styles.divider} />
      <StyledButton
        pressed={false}
        onClick={properties.onCopyStyles}
        title={t("toolbar.copy_styles")}
      >
        <Paintbrush2 />
      </StyledButton>
      <div class={styles.divider} />
      <StyledButton
        pressed={false}
        // onClick={(): void => {
        //   properties.onNumberFormatPicked(NumberFormats.CURRENCY_EUR);
        // }}
        title={t("toolbar.euro")}
      >
        <Euro />
      </StyledButton>
      <StyledButton
        pressed={false}
        // onClick={(): void => {
        //   properties.onNumberFormatPicked(NumberFormats.PERCENTAGE);
        // }}
        title={t("toolbar.percentage")}
      >
        <Percent />
      </StyledButton>
      <StyledButton
        pressed={false}
        // onClick={(): void => {
        //   properties.onNumberFormatPicked(
        //     decreaseDecimalPlaces(properties.numFmt)
        //   );
        // }}
        title={t("toolbar.decimal_places_decrease")}
      >
        <div><DecimalPlacesDecreaseIcon /></div>
      </StyledButton>
      <StyledButton
        pressed={false}
        // onClick={(): void => {
        //   properties.onNumberFormatPicked(
        //     increaseDecimalPlaces(properties.numFmt)
        //   );
        // }}
        title={t("toolbar.decimal_places_increase")}
      >
        <DecimalPlacesIncreaseIcon />
      </StyledButton>
      {/* //   <FormatMenu
    //     numFmt={properties.numFmt}
    //     onChange={(numberFmt): void => {
    //       properties.onNumberFormatPicked(numberFmt);
    //     }}
    //     onExited={(): void => {}}
    //     anchorOrigin={{
    //       horizontal: 20, // Aligning the menu to the middle of FormatButton
    //       vertical: "bottom",
    //     }}
    //   >*/
        <StyledButton

          pressed={false}
          
          title={t("toolbar.format_number")}
        >
          <div class={styles.format_menu}>{"123"}<ChevronDown /></div>
          
        </StyledButton>
    /*   </FormatMenu> */}
      <div class={styles.divider} />
      <StyledButton
        // pressed={properties.bold}
        // onClick={() => properties.onToggleBold(!properties.bold)}
        title={t("toolbar.bold")}
      >
        <Bold />
      </StyledButton>
      <StyledButton
        // pressed={properties.italic}
        // onClick={() => properties.onToggleItalic(!properties.italic)}
        title={t("toolbar.italic")}
      >
        <Italic />
      </StyledButton>
      <StyledButton
        // pressed={properties.underline}
        // onClick={() => properties.onToggleUnderline(!properties.underline)}
        title={t("toolbar.underline")}
      >
        <Underline />
      </StyledButton>
      <StyledButton
        // pressed={properties.strike}
        // onClick={() => properties.onToggleStrike(!properties.strike)}
        title={t("toolbar.strike_trough")}
      >
        <Strikethrough />
      </StyledButton>
      <div class={styles.divider} />
      <StyledButton
        pressed={false}
        title={t("toolbar.font_color")}
        // ref={fontColorButton}
        // underlinedColor={properties.fontColor}
        // onClick={() => setFontColorPickerOpen(true)}
      >
        <Type />
      </StyledButton>
      <StyledButton
        pressed={false}
        title={t("toolbar.fill_color")}
        // ref={fillColorButton}
        // underlinedColor={properties.fillColor}
        // onClick={() => setFillColorPickerOpen(true)}
      >
        <PaintBucket />
      </StyledButton>
      <div class={styles.divider} />
      <StyledButton
        // pressed={properties.horizontalAlign === "left"}
        // onClick={() =>
        //   properties.onToggleHorizontalAlign(
        //     properties.horizontalAlign === "left" ? "general" : "left"
        //   )
        // }
        title={t("toolbar.align_left")}
      >
        <AlignLeft />
      </StyledButton>
      <StyledButton
        // pressed={properties.horizontalAlign === "center"}
        // onClick={() =>
        //   properties.onToggleHorizontalAlign(
        //     properties.horizontalAlign === "center" ? "general" : "center"
        //   )
        // }
        title={t("toolbar.align_center")}
      >
        <AlignCenter />
      </StyledButton>
      <StyledButton
        // pressed={properties.horizontalAlign === "right"}
        // onClick={() =>
        //   properties.onToggleHorizontalAlign(
        //     properties.horizontalAlign === "right" ? "general" : "right"
        //   )
        // }
        title={t("toolbar.align_right")}
      >
        <AlignRight />
      </StyledButton>
      <StyledButton
        // pressed={properties.verticalAlign === "top"}
        // onClick={() =>
        //   properties.onToggleVerticalAlign(
        //     properties.verticalAlign === "top" ? "bottom" : "top"
        //   )
        // }
        title={t("toolbar.vertical_align_top")}
      >
        <ArrowUpToLine />
      </StyledButton>
      <StyledButton
        // pressed={properties.verticalAlign === "center"}
        // onClick={() =>
        //   properties.onToggleVerticalAlign(
        //     properties.verticalAlign === "center" ? "bottom" : "center"
        //   )
        // }
        title={t("toolbar.vertical_align_center")}
      >
        <ArrowMiddleFromLine />
      </StyledButton>
      <StyledButton
        // pressed={properties.verticalAlign === "bottom"}
        // onClick={() => properties.onToggleVerticalAlign("bottom")}
        title={t("toolbar.vertical_align_bottom")}
      >
        <ArrowDownToLine />
      </StyledButton>
      <div class={styles.divider} />
      <StyledButton
        pressed={false}
        // onClick={() => setBorderPickerOpen(true)}
        // ref={borderButton}
        title={t("toolbar.borders")}
      >
        <Grid2X2 />
      </StyledButton>
      {/* //   <ColorPicker
    //     color={properties.fontColor}
    //     onChange={(color): void => {
    //       properties.onTextColorPicked(color);
    //       setFontColorPickerOpen(false);
    //     }}
    //     anchorEl={fontColorButton}
    //     open={fontColorPickerOpen}
    //   />
    //   <ColorPicker
    //     color={properties.fillColor}
    //     onChange={(color): void => {
    //       properties.onFillColorPicked(color);
    //       setFillColorPickerOpen(false);
    //     }}
    //     anchorEl={fillColorButton}
    //     open={fillColorPickerOpen}
    //   />
    //   <BorderPicker
    //     onChange={(border): void => {
    //       properties.onBorderChanged(border);
    //       setBorderPickerOpen(false);
    //     }}
    //     anchorEl={borderButton}
    //     open={borderPickerOpen}
    //   /> */}
    </div>
  );
}

function StyledButton(props: {
  children: JSX.Element;
  title: string;
  onClick?: () => void;
  disabled?: boolean;
  pressed?: boolean;
  underlinedColor?: string;
}) {
  return (
    <button
      disabled={props.disabled || false}
      onClick={props.onClick}
      title={props.title}
      class={styles.button}
    >
      {props.children}
    </button>
  );
}

export default Toolbar;
