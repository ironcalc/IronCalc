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
  Triangle,
  X,
} from "lucide-react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { createAnchoredPortal } from "../createAnchoredPortal";
import "./icon-picker.css";

// Ordered list of all available icons for cycling.
const CYCLE_ICONS: {
  Icon: LucideIcon;
  name: string;
  filled?: boolean;
}[] = [
  { Icon: ArrowUp, name: "ArrowUp" },
  { Icon: ArrowUpRight, name: "ArrowAngleUp" },
  { Icon: ArrowRight, name: "ArrowRight" },
  { Icon: ArrowDownRight, name: "ArrowAngleDown" },
  { Icon: ArrowDown, name: "ArrowDown" },
  { Icon: ChevronUp, name: "TriangleUp" },
  { Icon: Minus, name: "FlatRectangle" },
  { Icon: ChevronDown, name: "TriangleDown" },
  { Icon: Check, name: "Check" },
  { Icon: CircleAlert, name: "Exclamation" },
  { Icon: X, name: "Cross" },
  { Icon: ThumbsUp, name: "ThumbsUp" },
  { Icon: ThumbsDown, name: "ThumbsDown" },
  { Icon: Circle, name: "Circle", filled: true },
  { Icon: Diamond, name: "Rhombus", filled: true },
  { Icon: Triangle, name: "Triangle", filled: true },
  { Icon: Flag, name: "Flag", filled: true },
  { Icon: Star, name: "Star", filled: true },
  { Icon: Heart, name: "Heart", filled: true },
];

export function iconSpecFor(name: string) {
  const result = CYCLE_ICONS.find((ic) => ic.name === name);
  if (!result) {
    console.warn(`Unknown icon name: ${name}`);
    return { Icon: X, filled: false };
  }
  return result;
}

interface IconPickerProps {
  value: string;
  color: string;
  onChange: (name: string) => void;
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
    if (!open) {
      return;
    }
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
        createAnchoredPortal(
          <div
            ref={panelRef}
            className="ic-icon-picker-panel"
            style={{ top: pos.top, left: pos.left }}
          >
            {CYCLE_ICONS.map(({ Icon, name, filled }) => (
              <button
                key={name}
                type="button"
                className={`ic-icon-picker-item${name === value ? " ic-icon-picker-item--selected" : ""}`}
                aria-label={name}
                onClick={() => {
                  onChange(name);
                  setOpen(false);
                }}
              >
                <Icon size={16} color={color} fill={filled ? color : "none"} />
              </button>
            ))}
          </div>,
          buttonRef.current,
        )}
    </>
  );
};

export default IconPicker;
