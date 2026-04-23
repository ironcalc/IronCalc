import { useEffect, useRef } from "react";
import { MenuItem } from "../Menu/MenuItem";
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
        <MenuItem
          key={tab.sheetId}
          checked={index === selectedIndex}
          icon={
            hasColors ? (
              <span
                className="ic-sheet-list-menu-color"
                style={{ backgroundColor: tab.color }}
              />
            ) : undefined
          }
          onClick={() => onSheetSelected(index)}
        >
          <span
            className={
              [
                index === selectedIndex && "ic-sheet-list-menu-name--selected",
                tab.state !== "visible" && "ic-sheet-list-menu-name--hidden",
              ]
                .filter(Boolean)
                .join(" ") || undefined
            }
          >
            {tab.name}
          </span>
        </MenuItem>
      ))}
    </div>
  );
};

export default SheetListMenu;
