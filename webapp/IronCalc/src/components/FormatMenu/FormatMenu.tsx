import type { FmtSettings } from "@ironcalc/wasm";
import { type ReactNode, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Menu } from "../Menu/Menu";
import { MenuDivider } from "../Menu/MenuDivider";
import { MenuItem } from "../Menu/MenuItem";
import FormatPicker from "./FormatPicker";
import { NumberFormats } from "./formatUtil";
import "./format-menu.css";

type FormatMenuProperties = {
  children: ReactNode;
  numFmt: string;
  onChange: (numberFmt: string) => void;
  formatOptions: FmtSettings;
};

interface FormatMenuItemsProperties {
  numFmt: string;
  onChange: (numberFmt: string) => void;
  formatOptions: FmtSettings;
  onOpenPicker: () => void;
  previousFocusedElementRef: React.RefObject<HTMLElement | null>;
}

function FormatMenuItems({
  numFmt,
  onChange,
  formatOptions,
  onOpenPicker,
  previousFocusedElementRef,
}: FormatMenuItemsProperties) {
  const { t } = useTranslation();

  function onSelect(s: string) {
    onChange(s);
    const prev = previousFocusedElementRef.current;
    requestAnimationFrame(() => prev?.focus());
  }

  const isAutoFormat = numFmt === NumberFormats.AUTO;
  const isNumberFormat = numFmt === formatOptions.number_fmt;
  const isPercentageFormat = numFmt === NumberFormats.PERCENTAGE;
  const isCurrencyEurosFormat = numFmt === NumberFormats.CURRENCY_EUR;
  const isCurrencyUsdFormat = numFmt === NumberFormats.CURRENCY_USD;
  const isCurrencyGbpFormat = numFmt === NumberFormats.CURRENCY_GBP;
  const isShortDateFormat = numFmt === formatOptions.short_date;
  const isLongDateFormat = numFmt === formatOptions.long_date;
  const isCustomFormat = !(
    isAutoFormat ||
    isNumberFormat ||
    isPercentageFormat ||
    isCurrencyEurosFormat ||
    isCurrencyUsdFormat ||
    isCurrencyGbpFormat ||
    isShortDateFormat ||
    isLongDateFormat
  );

  return (
    <>
      <MenuItem
        checked={isAutoFormat}
        onClick={() => onSelect(NumberFormats.AUTO)}
      >
        {t("toolbar.format_menu.auto")}
      </MenuItem>

      <MenuDivider />

      <MenuItem
        checked={isNumberFormat}
        onClick={() => onSelect(formatOptions.number_fmt)}
        secondaryText={formatOptions.number_example}
      >
        {t("toolbar.format_menu.number")}
      </MenuItem>
      <MenuItem
        checked={isPercentageFormat}
        onClick={() => onSelect(NumberFormats.PERCENTAGE)}
        secondaryText={t("toolbar.format_menu.percentage_example")}
      >
        {t("toolbar.format_menu.percentage")}
      </MenuItem>

      <MenuDivider />

      <MenuItem
        checked={isCurrencyEurosFormat}
        onClick={() => onSelect(NumberFormats.CURRENCY_EUR)}
        secondaryText={t("toolbar.format_menu.currency_eur_example")}
      >
        {t("toolbar.format_menu.currency_eur")}
      </MenuItem>
      <MenuItem
        checked={isCurrencyUsdFormat}
        onClick={() => onSelect(NumberFormats.CURRENCY_USD)}
        secondaryText={t("toolbar.format_menu.currency_usd_example")}
      >
        {t("toolbar.format_menu.currency_usd")}
      </MenuItem>
      <MenuItem
        checked={isCurrencyGbpFormat}
        onClick={() => onSelect(NumberFormats.CURRENCY_GBP)}
        secondaryText={t("toolbar.format_menu.currency_gbp_example")}
      >
        {t("toolbar.format_menu.currency_gbp")}
      </MenuItem>

      <MenuDivider />

      <MenuItem
        checked={isShortDateFormat}
        onClick={() => onSelect(formatOptions.short_date)}
        secondaryText={formatOptions.short_date_example}
      >
        {t("toolbar.format_menu.date_short")}
      </MenuItem>
      <MenuItem
        checked={isLongDateFormat}
        onClick={() => onSelect(formatOptions.long_date)}
        secondaryText={formatOptions.long_date_example}
      >
        {t("toolbar.format_menu.date_long")}
      </MenuItem>

      <MenuDivider />

      <MenuItem checked={isCustomFormat} onClick={onOpenPicker}>
        {t("toolbar.format_menu.custom")}
      </MenuItem>
    </>
  );
}

const FormatMenu = (properties: FormatMenuProperties) => {
  const [isPickerOpen, setPickerOpen] = useState(false);
  const previousFocusedElement = useRef<HTMLElement | null>(null);

  return (
    <div className="ic-format-menu-root">
      <Menu
        trigger={
          <div
            className="ic-format-menu-anchor"
            onMouseDownCapture={() => {
              previousFocusedElement.current =
                document.activeElement as HTMLElement | null;
            }}
          >
            {properties.children}
          </div>
        }
      >
        <FormatMenuItems
          numFmt={properties.numFmt}
          onChange={properties.onChange}
          formatOptions={properties.formatOptions}
          onOpenPicker={() => setPickerOpen(true)}
          previousFocusedElementRef={previousFocusedElement}
        />
      </Menu>

      <FormatPicker
        numFmt={properties.numFmt}
        onChange={(s) => {
          properties.onChange(s);
          requestAnimationFrame(() => previousFocusedElement.current?.focus());
        }}
        open={isPickerOpen}
        onClose={() => setPickerOpen(false)}
      />
    </div>
  );
};

export default FormatMenu;
