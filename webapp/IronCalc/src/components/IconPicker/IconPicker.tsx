import type { LucideIcon } from "lucide-react";
import {
  ArrowDown,
  ArrowDownRight,
  ArrowRight,
  ArrowUp,
  ArrowUpRight,
  Check,
  ChevronDown,
  ChevronUp,
  Circle,
  CircleAlert,
  Diamond,
  Flag,
  Heart,
  Minus,
  Star,
  ThumbsDown,
  ThumbsUp,
  X,
} from "lucide-react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import "./icon-picker.css";

// Ordered list of all available icons for cycling.
const CYCLE_ICONS: {
  Icon: LucideIcon;
  backendName: string;
  filled?: boolean;
}[] = [
  { Icon: ArrowUp, backendName: "ArrowUp" },
  { Icon: ArrowUpRight, backendName: "ArrowAngleUp" },
  { Icon: ArrowRight, backendName: "ArrowRight" },
  { Icon: ArrowDownRight, backendName: "ArrowAngleDown" },
  { Icon: ArrowDown, backendName: "ArrowDown" },
  { Icon: ChevronUp, backendName: "TriangleUp" },
  { Icon: Minus, backendName: "FlatRectangle" },
  { Icon: ChevronDown, backendName: "TriangleDown" },
  { Icon: Circle, backendName: "Circle", filled: true },
  { Icon: Diamond, backendName: "Rhombus", filled: true },
  { Icon: Flag, backendName: "Flag", filled: true },
  { Icon: Check, backendName: "Check" },
  { Icon: X, backendName: "Cross" },
  { Icon: CircleAlert, backendName: "Exclamation" },
  { Icon: Star, backendName: "Star", filled: true },
  { Icon: Heart, backendName: "Heart", filled: true },
  { Icon: ThumbsUp, backendName: "ThumbsUp", filled: true },
  { Icon: ThumbsDown, backendName: "ThumbsDown", filled: true },
];

export function iconSpecFor(backendName: string) {
  return (
    CYCLE_ICONS.find((ic) => ic.backendName === backendName) ?? CYCLE_ICONS[0]
  );
}

interface IconPickerProps {
  value: string;
  color: string;
  onChange: (backendName: string) => void;
}

const IconPicker = ({ value, color, onChange }: IconPickerProps) => {
  const [open, setOpen] = useState(false);
  const buttonRef = useRef<HTMLButtonElement | null>(null);
  const panelRef = useRef<HTMLDivElement | null>(null);
  const [pos, setPos] = useState({ top: -9999, left: -9999 });

  const spec = iconSpecFor(value);

  useLayoutEffect(() => {
    if (!open || !buttonRef.current || !panelRef.current) {
      return;
    }
    const anchor = buttonRef.current.getBoundingClientRect();
    const panel = panelRef.current;
    const panelWidth = panel.offsetWidth;
    const panelHeight = panel.offsetHeight;
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;
    const margin = 8;

    let left = anchor.left;
    let top = anchor.bottom + 4;

    if (left + panelWidth > viewportWidth - margin) {
      left = viewportWidth - panelWidth - margin;
    }
    if (top + panelHeight > viewportHeight - margin) {
      top = anchor.top - panelHeight - 4;
    }
    setPos({ top, left });
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (
        !buttonRef.current?.contains(e.target as Node) &&
        !panelRef.current?.contains(e.target as Node)
      ) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  return (
    <>
      <button
        ref={buttonRef}
        type="button"
        className="ic-icon-picker-trigger"
        aria-label="Pick icon"
        onClick={() => setOpen((o) => !o)}
      >
        <spec.Icon
          size={16}
          color={color}
          fill={spec.filled ? color : "none"}
        />
      </button>
      {open &&
        createPortal(
          <div
            ref={panelRef}
            className="ic-icon-picker-panel"
            style={{ top: pos.top, left: pos.left }}
          >
            {CYCLE_ICONS.map(({ Icon, backendName, filled }) => (
              <button
                key={backendName}
                type="button"
                className={`ic-icon-picker-item${backendName === value ? " ic-icon-picker-item--selected" : ""}`}
                aria-label={backendName}
                onClick={() => {
                  onChange(backendName);
                  setOpen(false);
                }}
              >
                <Icon size={16} color={color} fill={filled ? color : "none"} />
              </button>
            ))}
          </div>,
          document.body,
        )}
    </>
  );
};

export default IconPicker;
