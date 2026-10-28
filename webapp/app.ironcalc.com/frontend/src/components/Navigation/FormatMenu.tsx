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
  AlignLeft,
  Bold,
  Brush,
  Hash,
  Italic,
  Layers,
  RemoveFormatting,
  Strikethrough,
  SwatchBook,
  TableCellsMerge,
  Type,
  Underline,
  WrapText,
} from "lucide-react";
import { type RefObject, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

export function FormatMenu(props: {
  ironCalcRef: RefObject<IronCalcHandle | null>;
  model: Model;
  isOpen: boolean;
  onOpen: () => void;
  onClose: () => void;
  onHover: () => void;
}) {
  const [anchorPosition, setAnchorPosition] = useState({ x: 0, y: 0 });
  const triggerRef = useRef<HTMLButtonElement>(null);
  const { t } = useTranslation();
  const { model } = props;

  const captureAnchor = () => {
    const rect = triggerRef.current?.getBoundingClientRect();
    if (rect) {
      setAnchorPosition({ x: rect.left, y: rect.bottom + 4 });
    }
  };

  const actions = () => props.ironCalcRef.current?.workbook;

  const { sheet, row, column } = model.getSelectedView();
  const style = model.getCellStyle(sheet, row, column).style;
  const wrapText = style.alignment?.wrap_text ?? false;
  const merge = actions()?.getMergeCellsState();
  const fmt = model.getFmtSettings();
  const numberFormats = [
    [NumberFormats.AUTO, "auto"],
    null,
    [fmt.number_fmt, "number", fmt.number_example],
    [NumberFormats.PERCENTAGE, "percentage", "10%"],
    null,
    [NumberFormats.CURRENCY_EUR, "currency_eur", "€"],
    [NumberFormats.CURRENCY_USD, "currency_usd", "$"],
    [NumberFormats.CURRENCY_GBP, "currency_gbp", "£"],
    null,
    [fmt.short_date, "date_short", fmt.short_date_example],
    [fmt.long_date, "date_long", fmt.long_date_example],
  ];

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
          submenu={
            <>
              {numberFormats.map((item, index) =>
                item === null ? (
                  // biome-ignore lint/suspicious/noArrayIndexKey: static list
                  <MenuDivider key={index} />
                ) : (
                  <MenuItem
                    key={item[1]}
                    checked={style.num_fmt === item[0]}
                    secondaryText={item[2]}
                    onClick={() => actions()?.setNumberFormat(item[0])}
                  >
                    {t(`file_bar.format_menu.number_formats.${item[1]}`)}
                  </MenuItem>
                ),
              )}
              <MenuDivider />
              <MenuItem
                checked={
                  !numberFormats.some((item) => item?.[0] === style.num_fmt)
                }
                onClick={() => actions()?.openCustomNumberFormat()}
              >
                {t("file_bar.format_menu.number_formats.custom")}
              </MenuItem>
            </>
          }
        >
          {t("file_bar.format_menu.number_format")}
        </MenuItemWithSubmenu>
        <MenuDivider />
        <MenuItemWithSubmenu
          icon={<Type />}
          submenu={
            <>
              <MenuItem icon={<Bold />} onClick={() => actions()?.toggleBold()}>
                {t("file_bar.format_menu.bold")}
              </MenuItem>
              <MenuItem
                icon={<Italic />}
                onClick={() => actions()?.toggleItalic()}
              >
                {t("file_bar.format_menu.italic")}
              </MenuItem>
              <MenuItem
                icon={<Underline />}
                onClick={() => actions()?.toggleUnderline()}
              >
                {t("file_bar.format_menu.underline")}
              </MenuItem>
              <MenuItem
                icon={<Strikethrough />}
                onClick={() => actions()?.toggleStrikethrough()}
              >
                {t("file_bar.format_menu.strikethrough")}
              </MenuItem>
            </>
          }
        >
          {t("file_bar.format_menu.text")}
        </MenuItemWithSubmenu>
        <MenuItemWithSubmenu
          icon={<ALargeSmall />}
          submenu={[8, 9, 10, 11, 12, 14, 18, 24, 36].map((size) => (
            <MenuItem
              key={size}
              checked={style.font.sz === size}
              onClick={() => actions()?.setFontSize(size)}
            >
              {size}
            </MenuItem>
          ))}
        >
          {t("file_bar.format_menu.font_size")}
        </MenuItemWithSubmenu>
        <MenuDivider />
        <MenuItemWithSubmenu
          icon={<AlignLeft />}
          submenu={
            <>
              {(["left", "center", "right"] as const).map((alignment) => (
                <MenuItem
                  key={alignment}
                  onClick={() => actions()?.setHorizontalAlignment(alignment)}
                >
                  {t(`file_bar.format_menu.align_${alignment}`)}
                </MenuItem>
              ))}
              <MenuDivider />
              {(
                [
                  ["top", "align_top"],
                  ["center", "align_middle"],
                  ["bottom", "align_bottom"],
                ] as const
              ).map(([alignment, label]) => (
                <MenuItem
                  key={alignment}
                  onClick={() => actions()?.setVerticalAlignment(alignment)}
                >
                  {t(`file_bar.format_menu.${label}`)}
                </MenuItem>
              ))}
            </>
          }
        >
          {t("file_bar.format_menu.alignment")}
        </MenuItemWithSubmenu>
        <MenuItem
          icon={<WrapText />}
          onClick={() => actions()?.setWrapText(!wrapText)}
        >
          {t(
            wrapText
              ? "file_bar.format_menu.overflow_text"
              : "file_bar.format_menu.wrap_text",
          )}
        </MenuItem>
        <MenuItemWithSubmenu
          icon={<TableCellsMerge />}
          submenu={
            <>
              <MenuItem
                disabled={!merge?.canMerge}
                onClick={() => actions()?.mergeCells("merge")}
              >
                {t("file_bar.format_menu.merge_menu.merge")}
              </MenuItem>
              <MenuItem
                disabled={!merge?.canMerge}
                onClick={() => actions()?.mergeCells("merge_center")}
              >
                {t("file_bar.format_menu.merge_menu.merge_center")}
              </MenuItem>
              <MenuItem
                disabled={!merge?.canMergeAcross}
                onClick={() => actions()?.mergeCells("merge_across")}
              >
                {t("file_bar.format_menu.merge_menu.merge_across")}
              </MenuItem>
              <MenuItem
                disabled={!merge?.canMergeDown}
                onClick={() => actions()?.mergeCells("merge_down")}
              >
                {t("file_bar.format_menu.merge_menu.merge_down")}
              </MenuItem>
              <MenuDivider />
              <MenuItem
                disabled={!merge?.canUnmerge}
                onClick={() => actions()?.mergeCells("unmerge")}
              >
                {t("file_bar.format_menu.merge_menu.unmerge")}
              </MenuItem>
            </>
          }
        >
          {t("file_bar.format_menu.merge_cells")}
        </MenuItemWithSubmenu>
        <MenuDivider />
        <MenuItem
          icon={<Brush />}
          onClick={() => actions()?.openConditionalFormatting()}
        >
          {t("file_bar.format_menu.conditional_formatting")}
        </MenuItem>
        <MenuItem icon={<SwatchBook />} onClick={() => actions()?.openThemes()}>
          {t("file_bar.format_menu.themes")}
        </MenuItem>
        <MenuItem
          icon={<Layers />}
          onClick={() => actions()?.openNamedStyles()}
        >
          {t("file_bar.format_menu.cell_styles")}
        </MenuItem>
        <MenuDivider />
        <MenuItem
          icon={<RemoveFormatting />}
          onClick={() => actions()?.clearFormatting()}
        >
          {t("file_bar.format_menu.clear_formatting")}
        </MenuItem>
      </Menu>
    </div>
  );
}
