import {
  Button,
  type IronCalcHandle,
  Menu,
  MenuDivider,
  MenuItem,
  MenuItemWithSubmenu,
  type Model,
  NumberFormats,
} from "@ironcalc/workbook";
import {
  ALargeSmall,
  AlignCenter,
  AlignCenterHorizontal,
  AlignLeft,
  AlignRight,
  ArrowDownToLine,
  ArrowUpToLine,
  Bold,
  Hash,
  Italic,
  Minus,
  Plus,
  RemoveFormatting,
  Strikethrough,
  Type,
  Underline,
  WrapText,
} from "lucide-react";
import { type RefObject, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

const FONT_SIZES = [8, 9, 10, 11, 12, 14, 16, 18, 20, 24, 28, 36, 48];

export function FormatMenu(props: {
  // The widget's handle: the menu runs its actions through it
  ironCalcRef: RefObject<IronCalcHandle | null>;
  model: Model;
  isReadOnly: boolean;
  isOpen: boolean;
  onOpen: () => void;
  onClose: () => void;
  onHover: () => void;
}) {
  const [anchorPosition, setAnchorPosition] = useState({ x: 0, y: 0 });
  const triggerRef = useRef<HTMLButtonElement>(null);
  const { t } = useTranslation();
  const { model, isReadOnly } = props;

  const captureAnchor = () => {
    const rect = triggerRef.current?.getBoundingClientRect();
    if (rect) {
      setAnchorPosition({ x: rect.left, y: rect.bottom + 4 });
    }
  };

  const actions = () => props.ironCalcRef.current?.workbook;

  // The menu re-renders every time it opens, so the checks reflect the style
  // of the selected cell (the one the toolbar displays)
  const { sheet, row, column } = model.getSelectedView();
  const style = model.getCellStyle(sheet, row, column).style;
  const formatOptions = model.getFmtSettings();
  const horizontalAlign = style.alignment?.horizontal ?? "general";
  const verticalAlign = style.alignment?.vertical ?? "bottom";
  const wrapText = style.alignment?.wrap_text ?? false;

  const numberFormats: [string, string, string][] = [
    ["auto", NumberFormats.AUTO, ""],
    ["number", formatOptions.number_fmt, formatOptions.number_example],
    [
      "percentage",
      NumberFormats.PERCENTAGE,
      t("file_bar.format_menu.number.percentage_example"),
    ],
    ["currency_eur", NumberFormats.CURRENCY_EUR, "€"],
    ["currency_usd", NumberFormats.CURRENCY_USD, "$"],
    ["currency_gbp", NumberFormats.CURRENCY_GBP, "£"],
    ["date_short", formatOptions.short_date, formatOptions.short_date_example],
    ["date_long", formatOptions.long_date, formatOptions.long_date_example],
  ];

  const horizontalAlignments = [
    ["left", <AlignLeft key="left" />],
    ["center", <AlignCenter key="center" />],
    ["right", <AlignRight key="right" />],
  ] as const;

  const verticalAlignments = [
    ["top", <ArrowUpToLine key="top" />],
    ["center", <AlignCenterHorizontal key="center" />],
    ["bottom", <ArrowDownToLine key="bottom" />],
  ] as const;

  // The font flags are absent (not false) when unset, hence the coercion:
  // `checked` must be a boolean for the item to render its check mark
  const textStyles = [
    [
      "bold",
      !!style.font.b,
      <Bold key="bold" />,
      () => actions()?.toggleBold(),
    ],
    [
      "italic",
      !!style.font.i,
      <Italic key="italic" />,
      () => actions()?.toggleItalic(),
    ],
    [
      "underline",
      !!style.font.u,
      <Underline key="underline" />,
      () => actions()?.toggleUnderline(),
    ],
    [
      "strikethrough",
      !!style.font.strike,
      <Strikethrough key="strikethrough" />,
      () => actions()?.toggleStrikethrough(),
    ],
  ] as const;

  return (
    <div>
      <Button
        ref={triggerRef}
        variant="ghost"
        id="format-button"
        aria-haspopup="menu"
        aria-expanded={props.isOpen ? "true" : "false"}
        onClick={() => {
          captureAnchor();
          props.onOpen();
        }}
        onMouseEnter={() => {
          captureAnchor();
          props.onHover();
        }}
      >
        {t("file_bar.format_menu.button")}
      </Button>

      <Menu
        open={props.isOpen}
        onClose={props.onClose}
        anchorPosition={anchorPosition}
      >
        <MenuItemWithSubmenu
          icon={<Hash />}
          submenu={numberFormats.map(([key, format, example]) => (
            <MenuItem
              key={key}
              checked={style.num_fmt === format}
              disabled={isReadOnly}
              secondaryText={example}
              onClick={() => actions()?.setNumberFormat(format)}
            >
              {t(`file_bar.format_menu.number.${key}`)}
            </MenuItem>
          ))}
        >
          {t("file_bar.format_menu.number.title")}
        </MenuItemWithSubmenu>
        <MenuItemWithSubmenu
          icon={<Type />}
          submenu={textStyles.map(([key, checked, icon, onClick]) => (
            <MenuItem
              key={key}
              icon={icon}
              checked={checked}
              disabled={isReadOnly}
              onClick={onClick}
            >
              {t(`file_bar.format_menu.text.${key}`)}
            </MenuItem>
          ))}
        >
          {t("file_bar.format_menu.text.title")}
        </MenuItemWithSubmenu>
        <MenuItemWithSubmenu
          icon={<AlignLeft />}
          submenu={
            <>
              {horizontalAlignments.map(([alignment, icon]) => (
                <MenuItem
                  key={alignment}
                  icon={icon}
                  checked={horizontalAlign === alignment}
                  disabled={isReadOnly}
                  onClick={() =>
                    // Picking the current alignment again resets it, as the
                    // toolbar does
                    actions()?.setHorizontalAlignment(
                      horizontalAlign === alignment ? "general" : alignment,
                    )
                  }
                >
                  {t(`file_bar.format_menu.alignment.${alignment}`)}
                </MenuItem>
              ))}
              <MenuDivider />
              {verticalAlignments.map(([alignment, icon]) => (
                <MenuItem
                  key={alignment}
                  icon={icon}
                  checked={verticalAlign === alignment}
                  disabled={isReadOnly}
                  onClick={() => actions()?.setVerticalAlignment(alignment)}
                >
                  {t(`file_bar.format_menu.alignment.vertical_${alignment}`)}
                </MenuItem>
              ))}
            </>
          }
        >
          {t("file_bar.format_menu.alignment.title")}
        </MenuItemWithSubmenu>
        <MenuItem
          icon={<WrapText />}
          checked={wrapText}
          disabled={isReadOnly}
          onClick={() => actions()?.setWrapText(!wrapText)}
        >
          {t("file_bar.format_menu.wrap_text")}
        </MenuItem>
        <MenuItemWithSubmenu
          icon={<ALargeSmall />}
          submenu={
            <>
              <MenuItem
                icon={<Plus />}
                disabled={isReadOnly}
                onClick={() => actions()?.increaseFontSize(1)}
              >
                {t("file_bar.format_menu.font_size.increase")}
              </MenuItem>
              <MenuItem
                icon={<Minus />}
                disabled={isReadOnly}
                onClick={() => actions()?.increaseFontSize(-1)}
              >
                {t("file_bar.format_menu.font_size.decrease")}
              </MenuItem>
              <MenuDivider />
              {FONT_SIZES.map((size) => (
                <MenuItem
                  key={size}
                  checked={style.font.sz === size}
                  disabled={isReadOnly}
                  onClick={() => actions()?.setFontSize(size)}
                >
                  {size}
                </MenuItem>
              ))}
            </>
          }
        >
          {t("file_bar.format_menu.font_size.title")}
        </MenuItemWithSubmenu>
        <MenuDivider />
        <MenuItem
          icon={<RemoveFormatting />}
          disabled={isReadOnly}
          onClick={() => actions()?.clearFormatting()}
        >
          {t("file_bar.format_menu.clear_formatting")}
        </MenuItem>
      </Menu>
    </div>
  );
}
