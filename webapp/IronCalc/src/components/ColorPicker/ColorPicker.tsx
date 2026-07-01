import { Check, Plus } from "lucide-react";
import type React from "react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import AdvancedColorPicker from "../AdvancedColorPicker.tsx/AdvancedColorPicker";
import { createAnchoredPortal } from "../createAnchoredPortal";
import { getFocusableElements } from "../util";
import "./color-picker.css";
import type { Color, IronCalcTheme } from "@ironcalc/wasm";
import useAnchorPosition, { type Placement } from "./useAnchorPosition";
import useKeyDown from "./useKeyDown";
import {
  computeThemeGrid,
  getCheckColor,
  isWhiteColor,
  resolveColorToHex,
  standardColors,
  themeBaseColors,
} from "./util";

type ColorPickerProps = {
  color: Color;
  defaultColor: string;
  title: string;
  onChange: (color: Color) => void;
  onClose: () => void;
  anchorEl: React.RefObject<HTMLElement | null>;
  open: boolean;
  theme: IronCalcTheme;
  placement?: Placement;
};

const FALLBACK_COLOR = "#272525"; // --palette-common-black

const MAX_RECENT_COLORS = 29;

function colorsEqual(a: Color, b: Color): boolean {
  if (a === undefined || b === undefined) {
    return a === b;
  }
  if (Array.isArray(a) && Array.isArray(b)) {
    return a[0] === b[0] && a[1] === b[1];
  }
  if (typeof a === "string" && typeof b === "string") {
    return a.toUpperCase() === b.toUpperCase();
  }
  return false;
}

const ColorPicker = ({
  color,
  defaultColor,
  title,
  onChange,
  onClose,
  anchorEl,
  open,
  placement = "bottom",
  theme,
}: ColorPickerProps) => {
  const [selectedColor, setSelectedColor] = useState<Color>(color);
  const [isPickerOpen, setPickerOpen] = useState(false);

  const { panelRef, position } = useAnchorPosition(
    open && !isPickerOpen,
    anchorEl,
    placement,
  );

  const recentColors = useRef<{ color: Color; hex: string }[]>([]);
  const { t } = useTranslation();

  const themeColors = themeBaseColors(theme);

  const themeGrid = computeThemeGrid(theme);

  useEffect(() => {
    setSelectedColor(color);
  }, [color]);

  const { onKeyDown } = useKeyDown({
    onEscape: () => {
      setPickerOpen(false);
      onClose();
    },
    getFocusableElements: () => getFocusableElements(panelRef.current),
  });

  // focus the first button when the menu opens or when the advanced picker closes
  useEffect(() => {
    if (open && !isPickerOpen) {
      panelRef.current?.querySelector<HTMLButtonElement>("button")?.focus();
    }
  }, [open, isPickerOpen, panelRef]);

  // Close on presses outside the panel without swallowing them, so the
  // pressed element (e.g. another toolbar menu) reacts in the same click
  useEffect(() => {
    if (!open || isPickerOpen) {
      return;
    }

    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target as Node | null;

      if (!target) {
        return;
      }

      if (panelRef.current?.contains(target)) {
        return;
      }

      if (anchorEl.current?.contains(target)) {
        return;
      }

      onClose();
    };

    document.addEventListener("pointerdown", handlePointerDown, true);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown, true);
    };
  }, [open, isPickerOpen, onClose, anchorEl, panelRef]);

  const handleColorSelect = (colorValue: Color, displayHex: string) => {
    if (!recentColors.current.some((r) => colorsEqual(r.color, colorValue))) {
      recentColors.current = [
        { color: colorValue, hex: displayHex },
        ...recentColors.current,
      ].slice(0, MAX_RECENT_COLORS);
    }

    setSelectedColor(colorValue ?? FALLBACK_COLOR);
    onChange(colorValue);
    setPickerOpen(false);
  };

  const renderColorSwatch = (
    displayHex: string,
    colorValue: Color,
    row: number,
    col: number,
  ) => {
    const isSelected = colorsEqual(selectedColor, colorValue);

    const swatchClassName = [
      "ic-color-picker__swatch",
      "ic-color-picker__swatch--selectable",
      isWhiteColor(displayHex) ? "ic-color-picker__swatch--white" : "",
    ]
      .filter(Boolean)
      .join(" ");

    return (
      <button
        key={`r${row}c${col}`}
        type="button"
        className={swatchClassName}
        style={{ backgroundColor: displayHex }}
        onClick={() => handleColorSelect(colorValue, displayHex)}
        aria-label={displayHex}
        data-nav-row={row}
        data-nav-col={col}
      >
        {isSelected ? (
          <Check
            className="ic-color-picker__check-icon"
            style={{ color: getCheckColor(displayHex) }}
          />
        ) : null}
      </button>
    );
  };

  if (!open) {
    return null;
  }

  if (isPickerOpen) {
    return (
      <AdvancedColorPicker
        color={resolveColorToHex(selectedColor, theme) || FALLBACK_COLOR}
        onAccept={(hex) => handleColorSelect(hex, hex)}
        onCancel={() => setPickerOpen(false)}
        anchorEl={anchorEl}
        open={true}
      />
    );
  }

  return createAnchoredPortal(
    <div
      ref={panelRef}
      className="ic-color-picker"
      style={position}
      role="dialog"
      aria-label={t("color_picker.add")}
      onKeyDown={onKeyDown}
      onClick={(event) => {
        // Otherwise the sheet would grab the keyboard focus
        event.stopPropagation();
      }}
    >
      <div className="ic-color-picker__section">
        <button
          type="button"
          className="ic-color-picker__menu-item"
          onClick={() => handleColorSelect(defaultColor, defaultColor)}
          data-nav-row={0}
          data-nav-col={0}
        >
          <span
            className="ic-color-picker__menu-item-square"
            style={{ backgroundColor: defaultColor }}
            aria-hidden="true"
          />
          <span className="ic-color-picker__menu-item-text">{title}</span>
        </button>
      </div>

      <div className="ic-color-picker__divider" />

      <div className="ic-color-picker__section">
        <div className="ic-color-picker__label">
          {t("color_picker.themed_colors")}
        </div>

        <div className="ic-color-picker__color-list">
          {themeColors.map((hex, col) =>
            renderColorSwatch(hex, [col, 0], 1, col),
          )}
        </div>

        <div className="ic-color-picker__color-grid">
          {themeGrid.map((col, colIndex) => (
            <div
              className="ic-color-picker__color-grid-col"
              key={col.map((c) => c.hex).join("-")}
            >
              {col.map(({ hex, color }, toneIndex) =>
                renderColorSwatch(hex, color, 2 + toneIndex, colIndex),
              )}
            </div>
          ))}
        </div>
      </div>

      <div className="ic-color-picker__divider" />

      <div className="ic-color-picker__section">
        <div className="ic-color-picker__label">
          {t("color_picker.standard_colors")}
        </div>

        <div className="ic-color-picker__color-list">
          {standardColors.map((hex, col) =>
            renderColorSwatch(hex, hex, 7, col),
          )}
        </div>
      </div>

      <div className="ic-color-picker__divider" />

      <div className="ic-color-picker__section">
        <div className="ic-color-picker__label">{t("color_picker.recent")}</div>

        <div className="ic-color-picker__color-list">
          {recentColors.current.map(
            ({ color: recentColor, hex: recentHex }, col) => (
              <button
                key={recentHex}
                type="button"
                className={[
                  "ic-color-picker__swatch",
                  isWhiteColor(recentHex)
                    ? "ic-color-picker__swatch--white"
                    : "",
                ]
                  .filter(Boolean)
                  .join(" ")}
                style={{ backgroundColor: recentHex }}
                onClick={() => handleColorSelect(recentColor, recentHex)}
                aria-label={recentHex}
                data-nav-row={8}
                data-nav-col={col}
              />
            ),
          )}

          <button
            type="button"
            className="ic-color-picker__plus-button"
            onClick={() => setPickerOpen(true)}
            title={t("color_picker.add")}
            aria-label={t("color_picker.add")}
            data-nav-row={8}
            data-nav-col={recentColors.current.length}
          >
            <Plus />
          </button>
        </div>
      </div>
    </div>,
    anchorEl.current,
  );
};

export default ColorPicker;
