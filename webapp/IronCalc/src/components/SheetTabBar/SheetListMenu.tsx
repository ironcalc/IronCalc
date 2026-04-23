import { Check } from "lucide-react";
import { useContext, useEffect, useRef } from "react";
import { MenuContext } from "../Menu/Menu";
import type { SheetOptions } from "./types";
import "./sheet-list-menu.css";

function isWhiteColor(color: string): boolean {
  return ["#FFF", "#FFFFFF"].includes(color);
}

interface SheetListMenuProps {
  onSheetSelected: (index: number) => void;
  sheetOptionsList: SheetOptions[];
  selectedIndex: number;
}

const SheetListMenu = ({
  onSheetSelected,
  sheetOptionsList,
  selectedIndex,
}: SheetListMenuProps) => {
  const menuContext = useContext(MenuContext);
  const containerRef = useRef<HTMLDivElement>(null);
  const hasColors = sheetOptionsList.some((tab) => !isWhiteColor(tab.color));

  useEffect(() => {
    const frame = requestAnimationFrame(() => {
      const items = containerRef.current?.querySelectorAll<HTMLButtonElement>(
        '[role="menuitemradio"]',
      );
      items?.[selectedIndex]?.focus();
    });
    return () => cancelAnimationFrame(frame);
  }, [selectedIndex]);

  return (
    <div ref={containerRef}>
      {sheetOptionsList.map((tab, index) => (
        <button
          key={tab.sheetId}
          role="menuitemradio"
          aria-checked={index === selectedIndex}
          className="ic-menu-item"
          onClick={() => {
            onSheetSelected(index);
            menuContext?.close();
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
