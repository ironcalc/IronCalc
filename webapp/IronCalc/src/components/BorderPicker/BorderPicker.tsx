import {
  type BorderOptions,
  BorderStyle,
  BorderType,
  type Color,
  type IronCalcTheme,
} from "@ironcalc/wasm";
import {
  Grid2X2 as BorderAllIcon,
  ChevronRight,
  LineStyle,
  PencilLine,
} from "lucide-react";
import type React from "react";
import { useEffect, useRef, useState } from "react";
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
  BorderTopIcon,
} from "../../icons";
import { IconButton } from "../Button/IconButton";
import ColorPicker from "../ColorPicker/ColorPicker";
import { MenuItem, MenuItemWithSubmenu } from "../Menu/MenuItem";
import { useMenuPosition } from "../utils/useMenuPosition";
import "./border-picker.css";
import { STYLE_OPTIONS } from "./LineStylePicker";

type BorderPickerProps = {
  onChange: (border: BorderOptions) => void;
  onClose: () => void;
  anchorEl: React.RefObject<HTMLElement | null>;
  open: boolean;
  currentTheme: IronCalcTheme;
};

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
  currentTheme,
}: BorderPickerProps) {
  const { t } = useTranslation();

  const { menuRef: rootRef, position } = useMenuPosition(open, anchorEl);
  const borderColorButtonRef = useRef<HTMLButtonElement | null>(null);

  const [borderSelected, setBorderSelected] = useState<BorderType | null>(null);
  const [borderColor, setBorderColor] = useState<Color>("");
  const [borderStyle, setBorderStyle] = useState(BorderStyle.Thin);
  const [colorPickerOpen, setColorPickerOpen] = useState(false);

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
  }, [open, onClose, colorPickerOpen]);

  useEffect(() => {
    if (open) {
      return;
    }
    setBorderSelected(null);
    setBorderColor("");
    setBorderStyle(BorderStyle.Thin);
    setColorPickerOpen(false);
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
      // The line style submenu is portaled outside the picker
      if (target instanceof Element && target.closest(".ic-menu-wrapper")) {
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
  }, [open, onClose, colorPickerOpen, rootRef]);

  const applyBorder = (
    border: BorderType,
    color: Color,
    style: BorderStyle,
  ): void => {
    onChange({ color: color || undefined, style, border });
  };

  const toggleBorder = (
    value: BorderType,
    offValue: BorderType | null,
  ): void => {
    const next = borderSelected === value ? offValue : value;
    setBorderSelected(next);
    if (next !== null) {
      applyBorder(next, borderColor, borderStyle);
    }
  };

  if (!open || !anchorEl.current) {
    return null;
  }

  return (
    <div ref={rootRef} className="ic-border-picker" style={position}>
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
                    applyBorder(BorderType.None, borderColor, borderStyle);
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
            defaultColor=""
            defaultSwatchColor="var(--palette-sheet-default-text-color)"
            title={t("color_picker.default")}
            onChange={(color) => {
              setBorderColor(color);
              setColorPickerOpen(false);
              if (borderSelected) {
                applyBorder(borderSelected, color, borderStyle);
              }
            }}
            onClose={() => {
              setColorPickerOpen(false);
            }}
            anchorEl={borderColorButtonRef}
            open={colorPickerOpen}
            placement="right"
            theme={currentTheme}
          />
        </div>
        <MenuItemWithSubmenu
          icon={<LineStyle />}
          submenu={STYLE_OPTIONS.map((option) => (
            <MenuItem
              key={option.value}
              checked={borderStyle === option.value}
              onClick={() => {
                setBorderStyle(option.value);
                if (borderSelected) {
                  applyBorder(borderSelected, borderColor, option.value);
                }
              }}
            >
              <span className={`ic-line-preview ${option.previewClassName}`} />
            </MenuItem>
          ))}
        >
          {t("toolbar.borders.style")}
        </MenuItemWithSubmenu>
      </div>
    </div>
  );
}
