import type { FmtSettings } from "@ironcalc/wasm";
import { Check } from "lucide-react";
import { type ReactNode, useContext, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Menu, MenuContext } from "../Menu/Menu";
import { MenuDivider } from "../Menu/MenuDivider";
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
  const menu = useContext(MenuContext);

  function onSelect(s: string) {
    onChange(s);
    const prev = previousFocusedElementRef.current;
    menu?.close();
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
      <button
        className={`ic-menu-item ic-format-menu-item${isAutoFormat ? " is-selected" : ""}`}
        role="menuitemradio"
        aria-checked={isAutoFormat}
        onClick={() => onSelect(NumberFormats.AUTO)}
        type="button"
      >
        <span className="ic-format-menu-item-label">
          <Check className="ic-format-menu-item-check" />
          {t("toolbar.format_menu.auto")}
        </span>
      </button>

      <MenuDivider />

      <button
        className={`ic-menu-item ic-format-menu-item${isNumberFormat ? " is-selected" : ""}`}
        role="menuitemradio"
        aria-checked={isNumberFormat}
        onClick={() => onSelect(formatOptions.number_fmt)}
        type="button"
      >
        <span className="ic-format-menu-item-label">
          <Check className="ic-format-menu-item-check" />
          {t("toolbar.format_menu.number")}
        </span>
        <span className="ic-format-menu-item-example">
          {formatOptions.number_example}
        </span>
      </button>

      <button
        className={`ic-menu-item ic-format-menu-item${isPercentageFormat ? " is-selected" : ""}`}
        role="menuitemradio"
        aria-checked={isPercentageFormat}
        onClick={() => onSelect(NumberFormats.PERCENTAGE)}
        type="button"
      >
        <span className="ic-format-menu-item-label">
          <Check className="ic-format-menu-item-check" />
          {t("toolbar.format_menu.percentage")}
        </span>
        <span className="ic-format-menu-item-example">
          {t("toolbar.format_menu.percentage_example")}
        </span>
      </button>

      <MenuDivider />

      <button
        className={`ic-menu-item ic-format-menu-item${isCurrencyEurosFormat ? " is-selected" : ""}`}
        role="menuitemradio"
        aria-checked={isCurrencyEurosFormat}
        onClick={() => onSelect(NumberFormats.CURRENCY_EUR)}
        type="button"
      >
        <span className="ic-format-menu-item-label">
          <Check className="ic-format-menu-item-check" />
          {t("toolbar.format_menu.currency_eur")}
        </span>
        <span className="ic-format-menu-item-example">
          {t("toolbar.format_menu.currency_eur_example")}
        </span>
      </button>

      <button
        className={`ic-menu-item ic-format-menu-item${isCurrencyUsdFormat ? " is-selected" : ""}`}
        role="menuitemradio"
        aria-checked={isCurrencyUsdFormat}
        onClick={() => onSelect(NumberFormats.CURRENCY_USD)}
        type="button"
      >
        <span className="ic-format-menu-item-label">
          <Check className="ic-format-menu-item-check" />
          {t("toolbar.format_menu.currency_usd")}
        </span>
        <span className="ic-format-menu-item-example">
          {t("toolbar.format_menu.currency_usd_example")}
        </span>
      </button>

      <button
        className={`ic-menu-item ic-format-menu-item${isCurrencyGbpFormat ? " is-selected" : ""}`}
        role="menuitemradio"
        aria-checked={isCurrencyGbpFormat}
        onClick={() => onSelect(NumberFormats.CURRENCY_GBP)}
        type="button"
      >
        <span className="ic-format-menu-item-label">
          <Check className="ic-format-menu-item-check" />
          {t("toolbar.format_menu.currency_gbp")}
        </span>
        <span className="ic-format-menu-item-example">
          {t("toolbar.format_menu.currency_gbp_example")}
        </span>
      </button>

      <MenuDivider />

      <button
        className={`ic-menu-item ic-format-menu-item${isShortDateFormat ? " is-selected" : ""}`}
        role="menuitemradio"
        aria-checked={isShortDateFormat}
        onClick={() => onSelect(formatOptions.short_date)}
        type="button"
      >
        <span className="ic-format-menu-item-label">
          <Check className="ic-format-menu-item-check" />
          {t("toolbar.format_menu.date_short")}
        </span>
        <span className="ic-format-menu-item-example">
          {formatOptions.short_date_example}
        </span>
      </button>

      <button
        className={`ic-menu-item ic-format-menu-item${isLongDateFormat ? " is-selected" : ""}`}
        role="menuitemradio"
        aria-checked={isLongDateFormat}
        onClick={() => onSelect(formatOptions.long_date)}
        type="button"
      >
        <span className="ic-format-menu-item-label">
          <Check className="ic-format-menu-item-check" />
          {t("toolbar.format_menu.date_long")}
        </span>
        <span className="ic-format-menu-item-example">
          {formatOptions.long_date_example}
        </span>
      </button>

      <MenuDivider />

      <button
        className={`ic-menu-item ic-format-menu-item${isCustomFormat ? " is-selected" : ""}`}
        role="menuitemradio"
        aria-checked={isCustomFormat}
        onClick={() => {
          menu?.close();
          onOpenPicker();
        }}
        type="button"
      >
        <span className="ic-format-menu-item-label">
          <Check className="ic-format-menu-item-check" />
          {t("toolbar.format_menu.custom")}
        </span>
      </button>
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
