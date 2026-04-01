import type { FmtSettings } from "@ironcalc/wasm";
import { Check } from "lucide-react";
import {
  type KeyboardEvent as ReactKeyboardEvent,
  type ReactNode,
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import FormatPicker from "./FormatPicker";
import { NumberFormats } from "./formatUtil";
import "./format-menu.css";

type FormatMenuProperties = {
  children: ReactNode;
  numFmt: string;
  onChange: (numberFmt: string) => void;
  onExited: () => void;
  formatOptions: FmtSettings;
};

const FormatMenu = (properties: FormatMenuProperties) => {
  const { t } = useTranslation();
  const [isMenuOpen, setMenuOpen] = useState(false);
  const [isPickerOpen, setPickerOpen] = useState(false);
  const [menuStyle, setMenuStyle] = useState<{
    left?: number;
    top?: number;
  }>({});

  const anchorElement = useRef<HTMLDivElement>(null);
  const menuElement = useRef<HTMLDivElement>(null);

  const formatOptions = properties.formatOptions;

  const focusMenuItem = useCallback((index: number) => {
    const items =
      menuElement.current?.querySelectorAll<HTMLButtonElement>(
        ":scope > button",
      );

    if (!items || items.length === 0) {
      return;
    }

    const safeIndex = Math.max(0, Math.min(index, items.length - 1));
    items[safeIndex]?.focus();
  }, []);

  const focusSelectedOrFirstItem = useCallback(() => {
    const items =
      menuElement.current?.querySelectorAll<HTMLButtonElement>(
        ":scope > button",
      );

    if (!items || items.length === 0) {
      return;
    }

    const selectedIndex = Array.from(items).findIndex((item) =>
      item.classList.contains("is-selected"),
    );

    if (selectedIndex >= 0) {
      items[selectedIndex]?.focus();
      return;
    }

    items[0]?.focus();
  }, []);

  const onSelect = useCallback(
    (s: string) => {
      properties.onChange(s);
      setMenuOpen(false);
      anchorElement.current?.focus();
    },
    [properties.onChange],
  );

  const onMenuKeyDown = useCallback(
    (event: ReactKeyboardEvent<HTMLDivElement>) => {
      const items =
        menuElement.current?.querySelectorAll<HTMLButtonElement>(
          ":scope > button",
        );

      if (!items || items.length === 0) {
        return;
      }

      const itemsArray = Array.from(items);
      const currentIndex = itemsArray.indexOf(
        document.activeElement as HTMLButtonElement,
      );

      switch (event.key) {
        case "Escape":
          event.preventDefault();
          setMenuOpen(false);
          anchorElement.current?.focus();
          break;
        case "ArrowDown":
          event.preventDefault();
          if (currentIndex < 0) {
            focusSelectedOrFirstItem();
          } else {
            focusMenuItem((currentIndex + 1) % items.length);
          }
          break;
        case "ArrowUp":
          event.preventDefault();
          if (currentIndex < 0) {
            focusSelectedOrFirstItem();
          } else {
            focusMenuItem((currentIndex - 1 + items.length) % items.length);
          }
          break;
        case "Home":
          event.preventDefault();
          focusMenuItem(0);
          break;
        case "End":
          event.preventDefault();
          focusMenuItem(items.length - 1);
          break;
        case "Tab":
          setMenuOpen(false);
          break;
        default:
          break;
      }
    },
    [focusMenuItem, focusSelectedOrFirstItem],
  );

  useEffect(() => {
    if (!isMenuOpen) {
      return;
    }

    const onDocumentPointerDown = (event: PointerEvent): void => {
      const target = event.target as Node | null;

      if (
        anchorElement.current?.contains(target) ||
        menuElement.current?.contains(target)
      ) {
        return;
      }

      setMenuOpen(false);
    };

    document.addEventListener("pointerdown", onDocumentPointerDown, true);

    return () => {
      document.removeEventListener("pointerdown", onDocumentPointerDown, true);
    };
  }, [isMenuOpen]);

  useLayoutEffect(() => {
    if (!isMenuOpen || !anchorElement.current) {
      return;
    }

    const updateMenuPosition = (): void => {
      const rect = anchorElement.current?.getBoundingClientRect();

      if (!rect) {
        return;
      }

      setMenuStyle({
        left: rect.left,
        top: rect.bottom,
      });
    };

    updateMenuPosition();

    requestAnimationFrame(() => {
      focusSelectedOrFirstItem();
    });

    window.addEventListener("resize", updateMenuPosition);
    window.addEventListener("scroll", updateMenuPosition, true);

    return () => {
      window.removeEventListener("resize", updateMenuPosition);
      window.removeEventListener("scroll", updateMenuPosition, true);
    };
  }, [isMenuOpen, focusSelectedOrFirstItem]);

  const isAutoFormat = properties.numFmt === NumberFormats.AUTO;
  const isNumberFormat = properties.numFmt === formatOptions.number_fmt;
  const isPercentageFormat = properties.numFmt === NumberFormats.PERCENTAGE;
  const isCurrencyEurosFormat =
    properties.numFmt === NumberFormats.CURRENCY_EUR;
  const isCurrencyUsdFormat = properties.numFmt === NumberFormats.CURRENCY_USD;
  const isCurrencyGbpFormat = properties.numFmt === NumberFormats.CURRENCY_GBP;
  const isShortDateFormat = properties.numFmt === formatOptions.short_date;
  const isLongDateFormat = properties.numFmt === formatOptions.long_date;

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
    <div className="ic-format-menu-root">
      {/** biome-ignore lint/a11y/useSemanticElements: FIXME */}
      <div
        className="ic-format-menu-anchor"
        onClick={(): void => setMenuOpen(true)}
        onKeyDown={(event): void => {
          if (
            event.key === "Enter" ||
            event.key === " " ||
            event.key === "ArrowDown"
          ) {
            event.preventDefault();
            setMenuOpen(true);
          }
        }}
        ref={anchorElement}
        role="button"
        tabIndex={0}
        aria-haspopup="menu"
        aria-expanded={isMenuOpen}
      >
        {properties.children}
      </div>

      {isMenuOpen ? (
        <div
          className="ic-format-menu"
          ref={menuElement}
          style={menuStyle}
          role="menu"
          aria-label={t("toolbar.format_number")}
          onKeyDown={onMenuKeyDown}
        >
          <button
            className={isAutoFormat ? "is-selected" : undefined}
            onClick={(): void => onSelect(NumberFormats.AUTO)}
            type="button"
          >
            <span>
              <Check />
              {t("toolbar.format_menu.auto")}
            </span>
          </button>

          <div className="ic-format-menu-divider" />

          <button
            className={isNumberFormat ? "is-selected" : undefined}
            onClick={(): void => onSelect(formatOptions.number_fmt)}
            type="button"
          >
            <span>
              <Check />
              {t("toolbar.format_menu.number")}
            </span>
            <span>{formatOptions.number_example}</span>
          </button>

          <button
            className={isPercentageFormat ? "is-selected" : undefined}
            onClick={(): void => onSelect(NumberFormats.PERCENTAGE)}
            type="button"
          >
            <span>
              <Check />
              {t("toolbar.format_menu.percentage")}
            </span>
            <span>{t("toolbar.format_menu.percentage_example")}</span>
          </button>

          <div className="ic-format-menu-divider" />

          <button
            className={isCurrencyEurosFormat ? "is-selected" : undefined}
            onClick={(): void => onSelect(NumberFormats.CURRENCY_EUR)}
            type="button"
          >
            <span>
              <Check />
              {t("toolbar.format_menu.currency_eur")}
            </span>
            <span>{t("toolbar.format_menu.currency_eur_example")}</span>
          </button>

          <button
            className={isCurrencyUsdFormat ? "is-selected" : undefined}
            onClick={(): void => onSelect(NumberFormats.CURRENCY_USD)}
            type="button"
          >
            <span>
              <Check />
              {t("toolbar.format_menu.currency_usd")}
            </span>
            <span>{t("toolbar.format_menu.currency_usd_example")}</span>
          </button>

          <button
            className={isCurrencyGbpFormat ? "is-selected" : undefined}
            onClick={(): void => onSelect(NumberFormats.CURRENCY_GBP)}
            type="button"
          >
            <span>
              <Check />
              {t("toolbar.format_menu.currency_gbp")}
            </span>
            <span>{t("toolbar.format_menu.currency_gbp_example")}</span>
          </button>

          <div className="ic-format-menu-divider" />

          <button
            className={isShortDateFormat ? "is-selected" : undefined}
            onClick={(): void => onSelect(formatOptions.short_date)}
            type="button"
          >
            <span>
              <Check />
              {t("toolbar.format_menu.date_short")}
            </span>
            <span>{formatOptions.short_date_example}</span>
          </button>

          <button
            className={isLongDateFormat ? "is-selected" : undefined}
            onClick={(): void => onSelect(formatOptions.long_date)}
            type="button"
          >
            <span>
              <Check />
              {t("toolbar.format_menu.date_long")}
            </span>
            <span>{formatOptions.long_date_example}</span>
          </button>

          <div className="ic-format-menu-divider" />

          <button
            className={isCustomFormat ? "is-selected" : undefined}
            onClick={(): void => setPickerOpen(true)}
            type="button"
          >
            <span>
              <Check />
              {t("toolbar.format_menu.custom")}
            </span>
          </button>
        </div>
      ) : null}

      <FormatPicker
        numFmt={properties.numFmt}
        onChange={onSelect}
        open={isPickerOpen}
        onClose={(): void => setPickerOpen(false)}
        onExited={properties.onExited}
      />
    </div>
  );
};

export default FormatMenu;
