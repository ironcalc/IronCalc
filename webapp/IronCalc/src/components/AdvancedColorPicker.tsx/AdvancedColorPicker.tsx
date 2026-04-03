import { Check } from "lucide-react";
import type { RefObject } from "react";
import { useEffect, useId, useLayoutEffect, useRef, useState } from "react";
import { HexColorInput, HexColorPicker } from "react-colorful";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { Button } from "../Button/Button";
import "./advanced-color-picker.css";
import { getFocusableElements } from "../util";
import { useKeyDown } from "./useKeyDown";

type AdvancedColorPickerProps = {
  color: string;
  onAccept: (color: string) => void;
  onCancel: () => void;
  anchorEl: RefObject<HTMLElement | null>;
  open: boolean;
};

type Position = {
  top: number;
  left: number;
};

const MAX_RECENT_COLORS = 14;

const AdvancedColorPicker = ({
  color,
  onAccept,
  onCancel,
  anchorEl,
  open,
}: AdvancedColorPickerProps) => {
  const [selectedColor, setSelectedColor] = useState<string>(color);
  const [position, setPosition] = useState<Position>({ top: 0, left: 0 });
  const recentColors = useRef<string[]>([]);
  const panelRef = useRef<HTMLDivElement | null>(null);
  const hexInputId = useId();
  const { t } = useTranslation();

  useEffect(() => {
    setSelectedColor(color);
  }, [color]);

  useEffect(() => {
    if (!open) {
      return;
    }

    panelRef.current?.querySelector<HTMLElement>("input, button")?.focus();
  }, [open]);

  // poor person's popover positioning logic
  useLayoutEffect(() => {
    if (!open || !anchorEl.current) {
      return;
    }

    const updatePosition = () => {
      const anchor = anchorEl.current;
      const panel = panelRef.current;

      if (!anchor || !panel) {
        return;
      }

      const anchorRect = anchor.getBoundingClientRect();
      const panelWidth = panel.offsetWidth;
      const panelHeight = panel.offsetHeight;
      const viewportWidth = window.innerWidth;
      const viewportHeight = window.innerHeight;

      // space between anchor and popup
      const offset = 4;

      // minimum margin from screen edges
      const margin = 8;

      let left = anchorRect.left - offset;
      let top = anchorRect.bottom + offset;

      // If we are too much on the right, clamp to the right edge
      if (left + panelWidth > viewportWidth - margin) {
        left = viewportWidth - panelWidth - margin;
      }

      // If we are too much on the left, clamp to the left edge
      if (left < margin) {
        left = margin;
      }

      // If we are too much on the bottom, show above the anchor
      if (top + panelHeight > viewportHeight - margin) {
        top = anchorRect.top - panelHeight - offset;
      }

      // If we are too much on the top, clamp to the top edge
      if (top < margin) {
        top = margin;
      }

      setPosition({ top, left });
    };

    updatePosition();

    window.addEventListener("resize", updatePosition);
    window.addEventListener("scroll", updatePosition, true);

    return () => {
      window.removeEventListener("resize", updatePosition);
      window.removeEventListener("scroll", updatePosition, true);
    };
  }, [open, anchorEl]);

  const { onKeyDown } = useKeyDown({
    onEscape: onCancel,
    getFocusableElements: () => getFocusableElements(panelRef.current),
  });

  const handleColorSelect = (newColor: string) => {
    if (!recentColors.current.includes(newColor)) {
      recentColors.current = [newColor, ...recentColors.current].slice(
        0,
        MAX_RECENT_COLORS,
      );
    }

    setSelectedColor(newColor);
    onAccept(newColor);
  };

  const isWhiteSwatch = selectedColor.toUpperCase() === "#FFFFFF";

  if (!open) {
    return null;
  }

  return createPortal(
    <div className="ic-menu-layer">
      <div className="ic-menu-backdrop" onClick={onCancel} aria-hidden="true" />

      <div
        ref={panelRef}
        className="ic-advanced-color-picker-panel"
        style={{
          top: `${position.top}px`,
          left: `${position.left}px`,
        }}
        role="dialog"
        aria-modal="true"
        aria-label={t("color_picker.title")}
        onKeyDown={onKeyDown}
      >
        {/** biome-ignore lint/a11y/noStaticElementInteractions: FIXME */}
        {/** biome-ignore lint/a11y/useKeyWithClickEvents: FIXME */}
        <div
          className="ic-advanced-color-picker"
          onClick={(event) => {
            // Otherwise the sheet would grab the keyboard focus
            event.stopPropagation();
          }}
        >
          <HexColorPicker
            color={selectedColor}
            onChange={(newColor): void => {
              setSelectedColor(newColor);
            }}
          />

          <div className="ic-advanced-color-picker-divider" />

          <div className="ic-advanced-color-picker-input-row">
            <div className="ic-advanced-color-picker-hex-wrapper">
              <label
                className="ic-advanced-color-picker-hex-label"
                htmlFor={hexInputId}
              >
                Hex
              </label>

              <div className="ic-advanced-color-picker-hex-input-box">
                <div className="ic-advanced-color-picker-hash-label">#</div>

                <HexColorInput
                  id={hexInputId}
                  className="ic-advanced-color-picker-hex-input"
                  color={selectedColor}
                  onChange={(newColor): void => {
                    setSelectedColor(newColor);
                  }}
                />
              </div>
            </div>

            <div
              className={[
                "ic-advanced-color-picker-swatch",
                isWhiteSwatch ? "ic-advanced-color-picker-swatch--white" : "",
              ]
                .filter(Boolean)
                .join(" ")}
              style={{
                backgroundColor: selectedColor,
                borderColor: isWhiteSwatch
                  ? "var(--palette-grey-300)"
                  : selectedColor,
              }}
              aria-hidden="true"
            />
          </div>

          <div className="ic-advanced-color-picker-divider" />

          <div className="ic-advanced-color-picker-buttons">
            <Button size="sm" variant="secondary" onClick={onCancel}>
              {t("color_picker.cancel")}
            </Button>

            <Button
              size="sm"
              startIcon={<Check />}
              onClick={(): void => {
                handleColorSelect(selectedColor);
                onCancel();
              }}
            >
              {t("color_picker.apply")}
            </Button>
          </div>
        </div>
      </div>
    </div>,
    document.body,
  );
};

export default AdvancedColorPicker;
