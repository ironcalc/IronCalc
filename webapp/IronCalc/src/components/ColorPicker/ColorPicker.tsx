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
import { createPortal } from "react-dom";
import useKeyDown from "./useKeyDown";
import { getCheckColor, isWhiteColor, mainColors, toneArrays } from "./util";

type ColorPickerProps = {
  color: string;
  defaultColor: string;
  title: string;
  onChange: (color: string) => void;
  onClose: () => void;
  anchorEl: React.RefObject<HTMLElement | null>;
  open: boolean;
  placement?: Placement;
};

const FALLBACK_COLOR = "#272525"; // --palette-common-black

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
}: ColorPickerProps) => {
  const [selectedColor, setSelectedColor] = useState<string>(color);
  const [isPickerOpen, setPickerOpen] = useState(false);
  const [position, setPosition] = useState({ top: 0, left: 0 });

  const recentColors = useRef<string[]>([]);
  const panelRef = useRef<HTMLDivElement | null>(null);
  const { t } = useTranslation();

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
    [onClose, anchorEl.current],
  );

  const handleColorSelect = (nextColor: string) => {
    if (!recentColors.current.includes(nextColor)) {
      recentColors.current = [nextColor, ...recentColors.current].slice(0, 14);
    }

    setSelectedColor(nextColor || FALLBACK_COLOR);
    onChange(nextColor);
    setPickerOpen(false);
  };

  const renderColorSwatch = (presetColor: string, row: number, col: number) => {
    const isSelected =
      selectedColor.toUpperCase() === presetColor.toUpperCase();

    const swatchClassName = [
      "ic-color-picker__swatch",
      "ic-color-picker__swatch--selectable",
      isWhiteColor(presetColor) ? "ic-color-picker__swatch--white" : "",
    ]
      .filter(Boolean)
      .join(" ");

    return (
      <button
        key={presetColor}
        type="button"
        className={swatchClassName}
        style={{ backgroundColor: presetColor }}
        onClick={() => handleColorSelect(presetColor)}
        aria-label={presetColor}
        data-nav-row={row}
        data-nav-col={col}
      >
        {isSelected ? (
          <Check
            className="ic-color-picker__check-icon"
            style={{ color: getCheckColor(presetColor) }}
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
        color={selectedColor}
        onAccept={handleColorSelect}
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
        aria-label={t("color_picker.add")}
        onKeyDown={onKeyDown}
        onPointerDown={onPointerDown}
        onClick={(event) => {
          // Otherwise the sheet would grab the keyboard focus
          event.stopPropagation();
        }}
      >
        <button
          type="button"
          className="ic-color-picker__menu-item"
          onClick={() => handleColorSelect(defaultColor)}
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

        <div className="ic-color-picker__divider" />

        <div className="ic-color-picker__colors-wrapper">
          <div className="ic-color-picker__color-list">
            {mainColors.map((presetColor, col) =>
              renderColorSwatch(presetColor, 1, col),
            )}
          </div>

          <div className="ic-color-picker__color-grid">
            {toneArrays.map((tones, col) => (
              <div
                className="ic-color-picker__color-grid-col"
                key={tones.join("-")}
              >
                {tones.map((presetColor, toneIndex) =>
                  renderColorSwatch(presetColor, 2 + toneIndex, col),
                )}
              </div>
            ))}
          </div>
        </div>

        <div className="ic-color-picker__divider" />

        <div className="ic-color-picker__recent-label">
          {t("color_picker.recent")}
        </div>

        <div className="ic-color-picker__recent-colors-list">
          {recentColors.current.length > 0 ? (
            recentColors.current.map((recentColor, col) => (
              <button
                key={recentColor}
                type="button"
                className={[
                  "ic-color-picker__swatch",
                  isWhiteColor(recentColor)
                    ? "ic-color-picker__swatch--white"
                    : "",
                ]
                  .filter(Boolean)
                  .join(" ")}
                style={{ backgroundColor: recentColor }}
                onClick={() => {
                  setSelectedColor(recentColor);
                  handleColorSelect(recentColor);
                }}
                aria-label={recentColor}
                data-nav-row={7}
                data-nav-col={col}
              />
            ))
          ) : (
            <div className="ic-color-picker__empty" />
          )}

          <button
            type="button"
            className="ic-color-picker__plus-button"
            onClick={() => setPickerOpen(true)}
            title={t("color_picker.add")}
            aria-label={t("color_picker.add")}
            data-nav-row={7}
            data-nav-col={recentColors.current.length}
          >
            <Plus />
          </button>
        </div>
      </div>
    </div>,
    document.body,
  );
};

export default ColorPicker;
