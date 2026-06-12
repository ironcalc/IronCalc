import { Check, Plus } from "lucide-react";
import React, {
  type CSSProperties,
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import AdvancedColorPicker from "../AdvancedColorPicker.tsx/AdvancedColorPicker";
import { getFocusableElements } from "../util";
import "./color-picker.css";
import type { Color, IronCalcTheme } from "@ironcalc/wasm";
import { createPortal } from "react-dom";
import { Tooltip } from "../Tooltip/Tooltip";
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

type Placement = "bottom" | "top" | "right" | "left";

function getMenuPosition(
  anchor: HTMLElement,
  panel: HTMLElement,
  placement: Placement,
) {
  const anchorRect = anchor.getBoundingClientRect();
  const panelWidth = panel.offsetWidth;
  const panelHeight = panel.offsetHeight;
  const viewportWidth = window.innerWidth;
  const viewportHeight = window.innerHeight;

  const offset = 4;
  const margin = 8;

  let left = 0;
  let top = 0;

  if (placement === "bottom") {
    left = anchorRect.left;
    top = anchorRect.bottom + offset;
  } else if (placement === "top") {
    left = anchorRect.left;
    top = anchorRect.top - panelHeight - offset;
  } else if (placement === "right") {
    // This is used in the BorderPicker to be aligned with the line picker
    left = anchorRect.right;
    top = anchorRect.top;
  } else {
    left = anchorRect.left - panelWidth - offset;
    top = anchorRect.top;
  }

  if (left + panelWidth > viewportWidth - margin) {
    left = viewportWidth - panelWidth - margin;
  }

  if (left < margin) {
    left = margin;
  }

  if (top + panelHeight > viewportHeight - margin) {
    top = viewportHeight - panelHeight - margin;
  }

  if (top < margin) {
    top = margin;
  }

  return { top, left };
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
  const [position, setPosition] = useState({ top: 0, left: 0 });

  const recentColors = useRef<{ color: Color; hex: string }[]>([]);
  const panelRef = useRef<HTMLDivElement | null>(null);
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
  }, [open, isPickerOpen]);

  useLayoutEffect(() => {
    if (!open || isPickerOpen) {
      return;
    }

    const anchor = anchorEl.current;
    const panel = panelRef.current;

    if (!anchor || !panel) {
      return;
    }

    const updatePosition = () => {
      setPosition(getMenuPosition(anchor, panel, placement));
    };

    updatePosition();
    window.addEventListener("resize", updatePosition);
    window.addEventListener("scroll", updatePosition, true);

    return () => {
      window.removeEventListener("resize", updatePosition);
      window.removeEventListener("scroll", updatePosition, true);
    };
  }, [anchorEl, open, isPickerOpen, placement]);

  const onPointerDown = useCallback(
    (event: React.PointerEvent) => {
      const target = event.target as Node | null;
      const panel = panelRef.current;
      const anchor = anchorEl.current;

      if (!target || !panel) {
        return;
      }

      if (panel.contains(target)) {
        return;
      }

      if (anchor?.contains(target)) {
        return;
      }

      onClose();
    },
    [onClose, anchorEl],
  );

  const handleColorSelect = (colorValue: Color, displayHex: string) => {
    if (!recentColors.current.some((r) => colorsEqual(r.color, colorValue))) {
      recentColors.current = [
        { color: colorValue, hex: displayHex },
        ...recentColors.current,
      ];
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

  return createPortal(
    <div className="ic-menu-layer">
      <div
        className="ic-menu-backdrop"
        onPointerDown={onClose}
        aria-hidden="true"
      />
      <div
        ref={panelRef}
        className="ic-color-picker"
        style={
          {
            top: `${position.top}px`,
            left: `${position.left}px`,
          } as CSSProperties
        }
        role="dialog"
        aria-modal="true"
        aria-label={t("color_picker.add")}
        onKeyDown={onKeyDown}
        onPointerDown={onPointerDown}
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

          <div className="ic-color-picker__colors-wrapper">
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
        </div>

        <div className="ic-color-picker__divider" />

        <div className="ic-color-picker__section">
          <div className="ic-color-picker__label">
            {t("color_picker.standard_colors")}
          </div>

          <div className="ic-color-picker__color-list">
            {standardColors.map((hex, col) =>
              renderColorSwatch(hex, hex, 8, col),
            )}
          </div>
        </div>

        <div className="ic-color-picker__divider" />

        <div className="ic-color-picker__section">
          <div className="ic-color-picker__label">
            {t("color_picker.recent")}
          </div>

          <div className="ic-color-picker__color-list">
            {recentColors.current.length > 0 ? (
              recentColors.current.map(
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
                    data-nav-row={7}
                    data-nav-col={col}
                  />
                ),
              )
            ) : (
              <div className="ic-color-picker__empty" />
            )}

            <Tooltip title={t("color_picker.add")} container={panelRef.current}>
              <button
                type="button"
                className="ic-color-picker__plus-button"
                onClick={() => setPickerOpen(true)}
                aria-label={t("color_picker.add")}
                data-nav-row={7}
                data-nav-col={recentColors.current.length}
              >
                <Plus />
              </button>
            </Tooltip>
          </div>
        </div>
      </div>
    </div>,
    document.body,
  );
};

export default ColorPicker;
