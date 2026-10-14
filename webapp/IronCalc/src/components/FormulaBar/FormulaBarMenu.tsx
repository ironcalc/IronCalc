import type { Model } from "@ironcalc/wasm";
import { Tag } from "lucide-react";
import {
  type ReactNode,
  useCallback,
  useEffect,
  useId,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { parseRangeInSheet } from "../Editor/util";
import "./formula-bar-menu.css";

type FormulaBarMenuProps = {
  children: ReactNode;
  onMenuOpenChange: (isOpen: boolean) => void;
  openDrawer: () => void;
  canEdit: boolean;
  model: Model;
  onUpdate: () => void;
};

const FormulaBarMenu = (properties: FormulaBarMenuProps) => {
  const { t } = useTranslation();
  const [isMenuOpen, setMenuOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLDivElement>(null);
  const popoverRef = useRef<HTMLDivElement>(null);
  const [popoverMaxHeight, setPopoverMaxHeight] = useState<number>();
  const menuId = useId();

  const definedNameList = properties.model.getDefinedNameList();

  const openMenu = useCallback((): void => {
    setMenuOpen(true);
    properties.onMenuOpenChange(true);
  }, [properties.onMenuOpenChange]);

  const closeMenu = useCallback((): void => {
    setMenuOpen(false);
    properties.onMenuOpenChange(false);
    triggerRef.current?.focus();
  }, [properties.onMenuOpenChange]);

  const toggleMenu = useCallback((): void => {
    if (isMenuOpen) {
      closeMenu();
    } else {
      openMenu();
    }
  }, [closeMenu, isMenuOpen, openMenu]);

  useEffect(() => {
    if (!isMenuOpen) {
      return;
    }

    function handlePointerDown(event: PointerEvent): void {
      if (!rootRef.current?.contains(event.target as Node)) {
        closeMenu();
      }
    }

    function handleKeyDown(event: KeyboardEvent): void {
      if (event.key === "Escape") {
        closeMenu();
      }
    }

    document.addEventListener("pointerdown", handlePointerDown, true);
    document.addEventListener("keydown", handleKeyDown, true);

    return () => {
      document.removeEventListener("pointerdown", handlePointerDown, true);
      document.removeEventListener("keydown", handleKeyDown, true);
    };
  }, [closeMenu, isMenuOpen]);

  // Keep the popover inside the window, scrolling when the list is longer
  useLayoutEffect(() => {
    if (!isMenuOpen) {
      return;
    }

    function updateMaxHeight(): void {
      const popover = popoverRef.current;
      if (!popover) {
        return;
      }
      const margin = 8; // Safety margin from viewport edges
      const top = popover.getBoundingClientRect().top;
      setPopoverMaxHeight(window.innerHeight - top - margin);
    }

    updateMaxHeight();
    window.addEventListener("resize", updateMaxHeight);
    return () => {
      window.removeEventListener("resize", updateMaxHeight);
    };
  }, [isMenuOpen]);

  return (
    <div className="ic-formula-bar-menu" ref={rootRef}>
      {/** biome-ignore lint/a11y/noStaticElementInteractions: FIXME */}
      {/** biome-ignore lint/a11y/useKeyWithClickEvents: FIXME */}
      <div
        ref={triggerRef}
        className="ic-formula-bar-menu-trigger"
        onClick={toggleMenu}
      >
        {properties.children}
      </div>

      {isMenuOpen ? (
        <div
          ref={popoverRef}
          id={menuId}
          className="ic-formula-bar-menu-popover"
          role="menu"
          style={{ maxHeight: popoverMaxHeight }}
        >
          {definedNameList.length > 0 ? (
            <>
              {definedNameList.map((definedName) => {
                // FIXME: Implement the WAI-ARIA menu pattern
                // (see Select.tsx).
                return (
                  <button
                    key={`${definedName.name}-${definedName.scope}`}
                    type="button"
                    className="ic-formula-bar-menu-item"
                    role="menuitem"
                    onClick={() => {
                      const formula = definedName.formula;
                      const range = parseRangeInSheet(
                        properties.model,
                        formula,
                      );

                      if (range) {
                        const [
                          sheetIndex,
                          rowStart,
                          columnStart,
                          rowEnd,
                          columnEnd,
                        ] = range;
                        properties.model.setSelectedSheet(sheetIndex);
                        properties.model.setSelectedCell(rowStart, columnStart);
                        properties.model.setSelectedRange(
                          rowStart,
                          columnStart,
                          rowEnd,
                          columnEnd,
                        );
                      }

                      properties.onUpdate();
                      closeMenu();
                    }}
                  >
                    <Tag className="ic-formula-bar-menu-item-icon" />
                    <span className="ic-formula-bar-menu-item-text">
                      {definedName.name}
                    </span>
                    <span className="ic-formula-bar-menu-item-example">
                      {definedName.formula}
                    </span>
                  </button>
                );
              })}
              <div className="ic-formula-bar-menu-divider" />
            </>
          ) : null}

          <button
            type="button"
            className="ic-formula-bar-menu-item"
            role="menuitem"
            disabled={!properties.canEdit}
            onClick={() => {
              properties.openDrawer();
              closeMenu();
            }}
          >
            <span className="ic-formula-bar-menu-item-text">
              {t("formula_bar.manage_named_ranges")}
            </span>
          </button>
        </div>
      ) : null}
    </div>
  );
};

export default FormulaBarMenu;
