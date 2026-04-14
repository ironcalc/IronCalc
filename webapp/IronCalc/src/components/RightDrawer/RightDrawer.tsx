import type { Model } from "@ironcalc/wasm";
import type { MouseEvent as ReactMouseEvent } from "react";
import { useCallback, useEffect, useState } from "react";
import NamedRanges from "./NamedRanges/NamedRanges";
import RegionalSettings from "./RegionalSettings/RegionalSettings";
import "./rightdrawer.css";
import { useTranslation } from "react-i18next";

// Default drawer width is duplicated in CSS in rightdrawer.css; keep in sync
const DEFAULT_DRAWER_WIDTH = 360;
const MIN_DRAWER_WIDTH = 300;
const MAX_DRAWER_WIDTH = 500;

const KEYBOARD_RESIZE_STEP = 16;

export type DrawerType = "namedRanges" | "regionalSettings";

interface RightDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  width: number;
  onWidthChange: (width: number) => void;
  model: Model;
  onUpdate: () => void;
  getSelectedArea: () => string;
  drawerType: DrawerType;
  // Regional settings props
  initialLocale: string;
  initialTimezone: string;
  initialLanguage: string;
  onSettingsSave: (locale: string, timezone: string, language: string) => void;
}

const RightDrawer = ({
  isOpen,
  onClose,
  width,
  onWidthChange,
  getSelectedArea,
  model,
  onUpdate,
  drawerType,
  initialLocale,
  initialTimezone,
  initialLanguage,
  onSettingsSave,
}: RightDrawerProps) => {
  const [drawerWidth, setDrawerWidth] = useState(width);
  const [isResizing, setIsResizing] = useState(false);

  const { t } = useTranslation();

  const handleMouseDown = useCallback((e: ReactMouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
  }, []);

  // FIXME: Because of my complicated (aka stupid) global logic it is hard for the separator
  // to receive keyboard focus (a11y issue)
  // You can reach it via Shift+Tab from the locale select,
  // but any redraw steals focus back to the sheet.
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLHRElement>) => {
      let nextWidth = drawerWidth;

      if (e.key === "ArrowLeft") {
        nextWidth = Math.min(
          MAX_DRAWER_WIDTH,
          drawerWidth + KEYBOARD_RESIZE_STEP,
        );
      } else if (e.key === "ArrowRight") {
        nextWidth = Math.max(
          MIN_DRAWER_WIDTH,
          drawerWidth - KEYBOARD_RESIZE_STEP,
        );
      } else if (e.key === "Home") {
        nextWidth = MIN_DRAWER_WIDTH;
      } else if (e.key === "End") {
        nextWidth = MAX_DRAWER_WIDTH;
      } else {
        return;
      }

      setDrawerWidth(nextWidth);
      onWidthChange(nextWidth);
    },
    [drawerWidth, onWidthChange],
  );

  useEffect(() => {
    if (!isResizing) {
      return;
    }

    // Prevent text selection during resize
    document.body.style.userSelect = "none";
    document.body.style.cursor = "col-resize";

    const handleMouseMove = (e: MouseEvent) => {
      const newWidth = window.innerWidth - e.clientX;
      const clampedWidth = Math.max(
        MIN_DRAWER_WIDTH,
        Math.min(MAX_DRAWER_WIDTH, newWidth),
      );
      setDrawerWidth(clampedWidth);
      onWidthChange(clampedWidth);
    };

    const handleMouseUp = () => {
      setIsResizing(false);
      document.body.style.userSelect = "";
      document.body.style.cursor = "";
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
      document.body.style.userSelect = "";
      document.body.style.cursor = "";
    };
  }, [isResizing, onWidthChange]);

  if (!isOpen) {
    return null;
  }

  const renderDrawerContent = () => {
    switch (drawerType) {
      case "regionalSettings":
        return (
          <RegionalSettings
            onClose={onClose}
            initialLocale={initialLocale}
            initialTimezone={initialTimezone}
            initialLanguage={initialLanguage}
            onSave={onSettingsSave}
          />
        );
      default:
        return (
          <NamedRanges
            onClose={onClose}
            model={model}
            onUpdate={onUpdate}
            getSelectedArea={getSelectedArea}
          />
        );
    }
  };

  return (
    <div
      className="ic-drawer-container"
      style={{ ["--ic-runtime-drawer-width" as string]: `${drawerWidth}px` }}
    >
      <hr
        className={`ic-drawer-resize-handle ${isResizing ? "ic-drawer-resize-handle--resizing" : ""}`}
        tabIndex={0}
        aria-label={t("right_drawer.resize_drawer")}
        aria-orientation="vertical"
        onMouseDown={handleMouseDown}
        onKeyDown={handleKeyDown}
      />
      <div className="ic-drawer-divider" />
      <div className="ic-drawer-content">{renderDrawerContent()}</div>
    </div>
  );
};

export default RightDrawer;
export { DEFAULT_DRAWER_WIDTH, MAX_DRAWER_WIDTH, MIN_DRAWER_WIDTH };
