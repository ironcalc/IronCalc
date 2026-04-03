import { Check, Plus } from "lucide-react";
import React, {
  type CSSProperties,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import AdvancedColorPicker from "./AdvancedColorPicker";
import "./color-picker.css";

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

const mainColors = [
  "#FFFFFF",
  "#272525",
  "#1B717E",
  "#3BB68A",
  "#8CB354",
  "#F8CD3C",
  "#F2994A",
  "#EC5753",
  "#523E93",
  "#3358B7",
];

const lightTones = [
  "#F5F5F5", // --palette-grey-50
  "#F2F2F2", // --palette-grey-100
  "#EEEEEE", // --palette-grey-200
  "#E0E0E0", // --palette-grey-300
  "#BDBDBD", // --palette-grey-400
];

const darkTones = [
  "#9E9E9E", // --palette-grey-500
  "#757575", // --palette-grey-600
  "#616161", // --palette-grey-700
  "#424242", // --palette-grey-800
  "#333333", // --palette-grey-900
];

const tealTones = ["#BBD4D8", "#82B1B8", "#498D98", "#1E5A63", "#224348"];
const greenTones = ["#C4E9DC", "#93D7BF", "#62C5A1", "#358A6C", "#2F5F4D"];
const limeTones = ["#DDE8CC", "#C0D5A1", "#A3C276", "#6E8846", "#4F5E38"];
const yellowTones = ["#FDF0C5", "#FBE394", "#F9D764", "#B99A36", "#7A682E"];
const orangeTones = ["#FBE0C9", "#F8C79B", "#F5AD6E", "#B5763F", "#785334"];
const redTones = ["#F9CDCB", "#F5A3A0", "#F07975", "#B14845", "#763937"];
const purpleTones = ["#CBC5DF", "#A095C4", "#7565A9", "#453672", "#382F51"];
const blueTones = ["#C2CDE9", "#8FA3D7", "#5D79C5", "#30498B", "#2C395F"];

const toneArrays = [
  lightTones,
  darkTones,
  tealTones,
  greenTones,
  limeTones,
  yellowTones,
  orangeTones,
  redTones,
  purpleTones,
  blueTones,
];

// This function checks if a color is light or dark.
// This is needed to determine the icon color, as it's not visible on light colors.
const isLightColor = (hex: string): boolean => {
  if (!hex.startsWith("#")) {
    return false;
  }

  const normalized =
    hex.length === 4
      ? `#${hex[1]}${hex[1]}${hex[2]}${hex[2]}${hex[3]}${hex[3]}`
      : hex;

  const n = parseInt(normalized.slice(1), 16);
  const r = (n >> 16) & 255;
  const g = (n >> 8) & 255;
  const b = n & 255;

  const luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
  return luminance > 160;
};

function isWhiteColor(color: string): boolean {
  const upper = color.toUpperCase();
  return upper === "#FFF" || upper === "#FFFFFF";
}

function getCheckColor(color: string): string {
  // --palette-common-black: #272525;
  return isLightColor(color) ? "#272525" : "#FFFFFF";
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

  useEffect(() => {
    if (!open || isPickerOpen) {
      return;
    }

    const handlePointerDown = (event: MouseEvent) => {
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
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setPickerOpen(false);
        onClose();
      }
    };

    document.addEventListener("pointerdown", handlePointerDown, true);
    document.addEventListener("keydown", handleKeyDown, true);

    return () => {
      document.removeEventListener("pointerdown", handlePointerDown, true);
      document.removeEventListener("keydown", handleKeyDown, true);
    };
  }, [anchorEl, isPickerOpen, onClose, open]);

  const handleColorSelect = (nextColor: string) => {
    if (!recentColors.current.includes(nextColor)) {
      recentColors.current = [nextColor, ...recentColors.current].slice(0, 14);
    }

    setSelectedColor(nextColor || FALLBACK_COLOR);
    onChange(nextColor);
    setPickerOpen(false);
  };

  const renderColorSwatch = (presetColor: string) => {
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

  return (
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
    >
      <button
        type="button"
        className="ic-color-picker__menu-item"
        onClick={() => handleColorSelect(defaultColor)}
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
          {mainColors.map(renderColorSwatch)}
        </div>

        <div className="ic-color-picker__color-grid">
          {toneArrays.map((tones) => (
            <div
              className="ic-color-picker__color-grid-col"
              key={tones.join("-")}
            >
              {tones.map(renderColorSwatch)}
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
          recentColors.current.map((recentColor) => (
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
        >
          <Plus />
        </button>
      </div>
    </div>
  );
};

export default ColorPicker;
