import { type BorderOptions, BorderStyle, BorderType } from "@ironcalc/wasm";
import {
  Grid2X2 as BorderAllIcon,
  ChevronRight,
  PencilLine,
} from "lucide-react";
import type React from "react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  BorderBottomIcon,
  BorderCenterHIcon,
  BorderCenterVIcon,
  BorderInnerIcon,
  BorderLeftIcon,
  BorderNoneIcon,
  BorderOuterIcon,
  BorderRightIcon,
  BorderStyleIcon,
  BorderTopIcon,
} from "../../icons";
import { IconButton } from "../Button/IconButton";
import ColorPicker from "../ColorPicker/ColorPicker";
import "./border-picker.css";
import LineStylePicker from "./LineStylePicker";

type BorderPickerProps = {
  onChange: (border: BorderOptions) => void;
  onClose: () => void;
  anchorEl: React.RefObject<HTMLElement | null>;
  open: boolean;
};

type Position = {
  top: number;
  left: number;
};

// --palette-common-black
const DEFAULT_BORDER_COLOR = "#272525";

const BORDER_BUTTONS = [
  {
    value: BorderType.All,
    labelKey: "toolbar.borders.all",
    icon: BorderAllIcon,
    offValue: null,
  },
  {
    value: BorderType.Inner,
    labelKey: "toolbar.borders.inner",
    icon: BorderInnerIcon,
    offValue: null,
  },
  {
    value: BorderType.CenterH,
    labelKey: "toolbar.borders.horizontal",
    icon: BorderCenterHIcon,
    offValue: null,
  },
  {
    value: BorderType.CenterV,
    labelKey: "toolbar.borders.vertical",
    icon: BorderCenterVIcon,
    offValue: null,
  },
  {
    value: BorderType.Outer,
    labelKey: "toolbar.borders.outer",
    icon: BorderOuterIcon,
    offValue: BorderType.None,
  },
  {
    value: BorderType.None,
    labelKey: "toolbar.borders.clear",
    icon: BorderNoneIcon,
    offValue: BorderType.None,
  },
  {
    value: BorderType.Top,
    labelKey: "toolbar.borders.top",
    icon: BorderTopIcon,
    offValue: BorderType.None,
  },
  {
    value: BorderType.Right,
    labelKey: "toolbar.borders.right",
    icon: BorderRightIcon,
    offValue: BorderType.None,
  },
  {
    value: BorderType.Bottom,
    labelKey: "toolbar.borders.bottom",
    icon: BorderBottomIcon,
    offValue: BorderType.None,
  },
  {
    value: BorderType.Left,
    labelKey: "toolbar.borders.left",
    icon: BorderLeftIcon,
    offValue: BorderType.None,
  },
] as const;

export default function BorderPicker({
  onChange,
  onClose,
  anchorEl,
  open,
}: BorderPickerProps) {
  const { t } = useTranslation();

  const rootRef = useRef<HTMLDivElement | null>(null);
  const borderColorButtonRef = useRef<HTMLButtonElement | null>(null);

  const [position, setPosition] = useState<Position | null>(null);
  const [borderSelected, setBorderSelected] = useState<BorderType | null>(null);
  const [borderColor, setBorderColor] = useState(DEFAULT_BORDER_COLOR);
  const [borderStyle, setBorderStyle] = useState(BorderStyle.Thin);
  const [colorPickerOpen, setColorPickerOpen] = useState(false);
  const [stylePickerOpen, setStylePickerOpen] = useState(false);

  useLayoutEffect(() => {
    if (!open || !anchorEl.current) {
      return;
    }

    const updatePosition = () => {
      if (!anchorEl.current) {
        return;
      }

      const bb = anchorEl.current.getBoundingClientRect();
      setPosition({
        top: bb.bottom + 4,
        left: bb.left - 4,
      });
    };

    updatePosition();
    window.addEventListener("resize", updatePosition);
    window.addEventListener("scroll", updatePosition, true);

    return () => {
      window.removeEventListener("resize", updatePosition);
      window.removeEventListener("scroll", updatePosition, true);
    };
  }, [anchorEl, open]);

  useEffect(() => {
    if (!borderSelected) {
      return;
    }

    onChange({
      color: borderColor,
      style: borderStyle,
      border: borderSelected,
    });
  }, [borderColor, borderStyle, borderSelected, onChange]);

  useEffect(() => {
    if (!open) {
      return;
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        if (colorPickerOpen) {
          setColorPickerOpen(false);
          return;
        }
        if (stylePickerOpen) {
          setStylePickerOpen(false);
          return;
        }
        onClose();
      }
      // Block everything for now
      event.preventDefault();
      event.stopPropagation();
    };

    document.addEventListener("keydown", handleKeyDown, true);
    return () => {
      document.removeEventListener("keydown", handleKeyDown, true);
    };
  }, [open, onClose, colorPickerOpen, stylePickerOpen]);

  useEffect(() => {
    if (open) {
      return;
    }
    setBorderSelected(null);
    setBorderColor(DEFAULT_BORDER_COLOR);
    setBorderStyle(BorderStyle.Thin);
    setColorPickerOpen(false);
    setStylePickerOpen(false);
  }, [open]);

  useEffect(() => {
    if (!open) {
      return;
    }

    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target as Node | null;
      if (target && rootRef.current?.contains(target)) {
        return;
      }
      if (colorPickerOpen) {
        return;
      }
      onClose();
    };

    document.addEventListener("pointerdown", handlePointerDown, true);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown, true);
    };
  }, [open, onClose, colorPickerOpen]);

  const toggleBorder = (
    value: BorderType,
    offValue: BorderType | null,
  ): void => {
    setBorderSelected((current) => (current === value ? offValue : value));
  };

  if (!open || !anchorEl.current) {
    return null;
  }

  return (
    <div
      ref={rootRef}
      className="ic-border-picker"
      style={
        position
          ? { top: `${position.top}px`, left: `${position.left}px` }
          : undefined
      }
    >
      <div className="ic-border-picker__borders">
        <div className="row">
          {BORDER_BUTTONS.slice(0, 5).map((button) => {
            const Icon = button.icon;
            const value = button.value;
            return (
              <IconButton
                key={value}
                pressed={borderSelected === value}
                aria-label={t(button.labelKey)}
                title={t(button.labelKey)}
                icon={<Icon />}
                onClick={() => toggleBorder(value, button.offValue)}
              />
            );
          })}
        </div>

        <div className="row">
          {BORDER_BUTTONS.slice(5).map((button) => {
            const Icon = button.icon;
            const value = button.value;
            return (
              <IconButton
                key={value}
                pressed={borderSelected === value}
                aria-label={t(button.labelKey)}
                title={t(button.labelKey)}
                icon={<Icon />}
                onClick={() => {
                  if (value === BorderType.None) {
                    setBorderSelected(BorderType.None);
                    return;
                  }
                  toggleBorder(value, button.offValue);
                }}
              />
            );
          })}
        </div>
      </div>

      <div className="divider" />

      <div className="ic-border-picker__menu">
        {/** biome-ignore lint/a11y/noStaticElementInteractions: FIXME */}
        {/** biome-ignore lint/a11y/useKeyWithClickEvents: FIXME */}
        <div
          className="ic-border-picker__submenu-anchor"
          onClick={() => setColorPickerOpen((prev) => !prev)}
        >
          <button
            ref={borderColorButtonRef}
            type="button"
            className="ic-border-picker__button"
          >
            <PencilLine />
            <span>{t("toolbar.borders.color")}</span>
            <ChevronRight />
          </button>
          <ColorPicker
            color={borderColor}
            defaultColor={DEFAULT_BORDER_COLOR}
            title={t("color_picker.default")}
            onChange={(color) => {
              setBorderColor(color);
              setColorPickerOpen(false);
            }}
            onClose={() => {
              setColorPickerOpen(false);
            }}
            anchorEl={borderColorButtonRef}
            open={colorPickerOpen}
            placement="right"
          />
        </div>
        {/* biome-ignore lint/a11y/noStaticElementInteractions: FIXME */}
        <div
          className="ic-border-picker__submenu-anchor"
          onMouseEnter={() => setStylePickerOpen(true)}
          onMouseLeave={() => setStylePickerOpen(false)}
        >
          <button type="button" className="ic-border-picker__button">
            <BorderStyleIcon />
            <span>{t("toolbar.borders.style")}</span>
            <ChevronRight />
          </button>

          {stylePickerOpen && (
            <LineStylePicker
              value={borderStyle}
              onSelect={(style) => {
                setBorderStyle(style);
                setStylePickerOpen(false);
              }}
            />
          )}
        </div>
      </div>
    </div>
  );
}
