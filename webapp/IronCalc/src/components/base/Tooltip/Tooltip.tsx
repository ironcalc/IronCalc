import { useTheme } from "@mui/material";
import { alpha } from "@mui/material/styles";
import {
  type ReactElement,
  type ReactNode,
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { createPortal } from "react-dom";

export type TooltipPlacement = "bottom" | "left" | "right" | "top";

const GAP = 4;
const TRANSITION_MS = 150;
const ENTER_DELAY_MS = 700;
const LEAVE_DELAY_MS = 0;
// If a tooltip was visible within this window (ms), the next one shows immediately.
const RECENT_TOOLTIP_MS = 500;

let lastTooltipVisibleAt = 0;

export interface TooltipProps {
  title: ReactNode;
  placement: TooltipPlacement;
  disableHoverListener: boolean;
  children: ReactElement;
}

function getTooltipPosition(
  placement: TooltipPlacement,
  triggerRect: DOMRect,
): { left: number; top: number; transform: string } {
  const { left, top, right, bottom, width, height } = triggerRect;
  const cx = left + width / 2;
  const cy = top + height / 2;

  switch (placement) {
    case "bottom":
      return { left: cx, top: bottom + GAP, transform: "translate(-50%, 0)" };
    case "top":
      return { left: cx, top: top - GAP, transform: "translate(-50%, -100%)" };
    case "left":
      return { left: left - GAP, top: cy, transform: "translate(-100%, -50%)" };
    case "right":
      return { left: right + GAP, top: cy, transform: "translate(0, -50%)" };
  }
}

export function Tooltip({
  title,
  placement,
  disableHoverListener,
  children,
}: TooltipProps) {
  const theme = useTheme();
  const triggerRef = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);
  const [exiting, setExiting] = useState(false);
  const [entered, setEntered] = useState(false);
  const [position, setPosition] = useState<{
    left: number;
    top: number;
    transform: string;
  } | null>(null);
  const enterTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const leaveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearTimers = useCallback(() => {
    if (enterTimerRef.current) {
      clearTimeout(enterTimerRef.current);
      enterTimerRef.current = null;
    }
    if (leaveTimerRef.current) {
      clearTimeout(leaveTimerRef.current);
      leaveTimerRef.current = null;
    }
  }, []);

  useEffect(() => () => clearTimers(), [clearTimers]);

  const handleEnter = useCallback(
    (_e: React.MouseEvent | React.FocusEvent) => {
      if (disableHoverListener) return;
      clearTimers();
      leaveTimerRef.current = null;
      const delay =
        Date.now() - lastTooltipVisibleAt < RECENT_TOOLTIP_MS
          ? 0
          : ENTER_DELAY_MS;
      enterTimerRef.current = setTimeout(() => {
        lastTooltipVisibleAt = Date.now();
        setExiting(false);
        setEntered(false);
        setOpen(true);
      }, delay);
    },
    [disableHoverListener, clearTimers],
  );

  const handleLeave = useCallback(
    (_e: React.MouseEvent | React.FocusEvent) => {
      clearTimers();
      enterTimerRef.current = null;
      if (!open) return;
      lastTooltipVisibleAt = Date.now();
      leaveTimerRef.current = setTimeout(() => {
        setOpen(false);
        setExiting(true);
      }, LEAVE_DELAY_MS);
    },
    [clearTimers, open],
  );

  useLayoutEffect(() => {
    if (!open || !triggerRef.current) return;
    const triggerRect = triggerRef.current.getBoundingClientRect();
    setPosition(getTooltipPosition(placement, triggerRect));
  }, [open, placement]);

  useEffect(() => {
    if (open && !entered) {
      const id = requestAnimationFrame(() => setEntered(true));
      return () => cancelAnimationFrame(id);
    }
  }, [open, entered]);

  const mounted = (open || exiting) && title != null && title !== "";
  const visible = open && entered;

  const handleTransitionEnd = useCallback(
    (e: React.TransitionEvent) => {
      if (e.propertyName === "opacity" && exiting) setExiting(false);
    },
    [exiting],
  );

  const tooltipStyle: React.CSSProperties = {
    position: "fixed",
    left: position?.left ?? -9999,
    top: position?.top ?? -9999,
    transform: position?.transform ?? "translate(-50%, 0)",
    zIndex: theme.zIndex.tooltip,
    fontFamily: "Inter, sans-serif",
    fontSize: 10,
    padding: "4px 8px",
    maxWidth: 300,
    backgroundColor: alpha(theme.palette.grey[900], 0.76),
    color: theme.palette.common.white,
    borderRadius: 4,
    pointerEvents: "none",
    boxSizing: "border-box",
    opacity: visible ? 1 : 0,
    transition: `opacity ${TRANSITION_MS}ms ease`,
  };

  const tooltipContent = mounted && (
    <div
      role="tooltip"
      style={tooltipStyle}
      onTransitionEnd={handleTransitionEnd}
    >
      {title}
    </div>
  );

  return (
    <>
      {/* biome-ignore lint/a11y/useSemanticElements: tooltip trigger wrapper, not a form group — fieldset would be wrong */}
      <div
        ref={triggerRef}
        role="group"
        onMouseEnter={handleEnter}
        onMouseLeave={handleLeave}
        onFocus={handleEnter}
        onBlur={handleLeave}
        style={{ display: "inline-flex" }}
      >
        {children}
      </div>
      {typeof document !== "undefined" &&
        createPortal(tooltipContent, document.body)}
    </>
  );
}
