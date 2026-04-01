import { Check } from "lucide-react";
import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import type { SheetOptions } from "./types";
import "./sheet-list-menu.css";

function isWhiteColor(color: string): boolean {
  return ["#FFF", "#FFFFFF"].includes(color);
}

interface SheetListMenuProps {
  open: boolean;
  onClose: () => void;
  anchorEl: HTMLButtonElement | null;
  onSheetSelected: (index: number) => void;
  sheetOptionsList: SheetOptions[];
  selectedIndex: number;
}

const SheetListMenu = (properties: SheetListMenuProps) => {
  const {
    open,
    onClose,
    anchorEl,
    onSheetSelected,
    sheetOptionsList,
    selectedIndex,
  } = properties;

  const [menuStyle, setMenuStyle] = useState<{
    left?: number;
    bottom?: number;
  }>({});

  const menuRef = useRef<HTMLDivElement>(null);

  const hasColors = sheetOptionsList.some((tab) => !isWhiteColor(tab.color));

  const getMenuItems = useCallback(() => {
    const items =
      menuRef.current?.querySelectorAll<HTMLButtonElement>(":scope > button");

    if (!items) {
      return [];
    }

    return Array.from(items).filter((item) => !item.disabled);
  }, []);

  const focusMenuItem = useCallback(
    (index: number) => {
      const items = getMenuItems();

      if (items.length === 0) {
        return;
      }

      const safeIndex = Math.max(0, Math.min(index, items.length - 1));
      items[safeIndex]?.focus();
    },
    [getMenuItems],
  );

  useLayoutEffect(() => {
    if (!open || !anchorEl) {
      return;
    }

    const updateMenuPosition = () => {
      const rect = anchorEl.getBoundingClientRect();

      setMenuStyle({
        left: rect.left,
        bottom: window.innerHeight - rect.top,
      });
    };

    updateMenuPosition();

    window.addEventListener("resize", updateMenuPosition);
    window.addEventListener("scroll", updateMenuPosition, true);

    return () => {
      window.removeEventListener("resize", updateMenuPosition);
      window.removeEventListener("scroll", updateMenuPosition, true);
    };
  }, [open, anchorEl]);

  useEffect(() => {
    if (!open) {
      return;
    }

    const frame = requestAnimationFrame(() => {
      focusMenuItem(selectedIndex);
    });

    return () => {
      cancelAnimationFrame(frame);
    };
  }, [open, selectedIndex, focusMenuItem]);

  useEffect(() => {
    if (!open) {
      return;
    }

    const onDocumentPointerDown = (event: PointerEvent) => {
      const path = event.composedPath();

      if (menuRef.current && path.includes(menuRef.current)) {
        return;
      }

      if (anchorEl && path.includes(anchorEl)) {
        return;
      }

      onClose();
    };

    document.addEventListener("pointerdown", onDocumentPointerDown, true);

    return () => {
      document.removeEventListener("pointerdown", onDocumentPointerDown, true);
    };
  }, [open, anchorEl, onClose]);

  if (!open) {
    return null;
  }

  return (
    <div
      className="ic-sheet-list-menu"
      ref={menuRef}
      style={menuStyle}
      role="menu"
      aria-label="Sheet list"
      onKeyDown={(event) => {
        const items = getMenuItems();

        if (items.length === 0) {
          return;
        }

        const currentIndex = items.indexOf(
          document.activeElement as HTMLButtonElement,
        );

        switch (event.key) {
          case "Escape":
            event.preventDefault();
            onClose();
            requestAnimationFrame(() => {
              anchorEl?.focus();
            });
            break;

          case "ArrowDown":
            event.preventDefault();
            if (currentIndex === -1) {
              focusMenuItem(0);
            } else {
              focusMenuItem((currentIndex + 1) % items.length);
            }
            break;

          case "ArrowUp":
            event.preventDefault();
            if (currentIndex === -1) {
              focusMenuItem(items.length - 1);
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
            onClose();
            break;

          default:
            break;
        }
      }}
    >
      {sheetOptionsList.map((tab, index) => (
        <button
          key={tab.sheetId}
          role="menuitemradio"
          aria-checked={index === selectedIndex}
          className="ic-sheet-list-menu-item"
          onClick={() => {
            onSheetSelected(index);
            onClose();
            requestAnimationFrame(() => {
              anchorEl?.focus();
            });
          }}
          type="button"
        >
          {index === selectedIndex ? (
            <Check className="ic-sheet-list-menu-check" />
          ) : (
            <div className="ic-sheet-list-menu-check-placeholder" />
          )}

          {hasColors ? (
            <div
              className="ic-sheet-list-menu-color"
              style={{ backgroundColor: tab.color }}
            />
          ) : null}

          <div
            className={`ic-sheet-list-menu-name${
              index === selectedIndex
                ? " ic-sheet-list-menu-name--selected"
                : ""
            }${tab.state === "visible" ? "" : " ic-sheet-list-menu-name--hidden"}`}
          >
            {tab.name}
          </div>
        </button>
      ))}
    </div>
  );
};

export default SheetListMenu;
