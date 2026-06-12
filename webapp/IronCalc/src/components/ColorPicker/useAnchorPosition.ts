import {
  type CSSProperties,
  type RefObject,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

export type Placement = "bottom" | "top" | "right";

function getPanelPosition(
  anchor: HTMLElement,
  panel: HTMLElement,
  placement: Placement,
) {
  const anchorRect = anchor.getBoundingClientRect();
  const panelWidth = panel.offsetWidth;
  const panelHeight = panel.offsetHeight;
  const viewportWidth = window.innerWidth;
  const viewportHeight = window.innerHeight;

  const offset = 4; // Distance between anchor and panel
  const margin = 8; // Safety margin from viewport edges

  let left: number;
  let top: number;

  if (placement === "right") {
    // Aligned with the anchor's top edge, used by submenus (e.g. BorderPicker)
    left = anchorRect.right;
    top = anchorRect.top;
  } else {
    // Preferred side by default, falls back to the other side when it has more space
    const spaceBelow = viewportHeight - anchorRect.bottom - margin;
    const spaceAbove = anchorRect.top - margin;
    const openBelow =
      placement === "top"
        ? spaceAbove < panelHeight + offset && spaceBelow > spaceAbove
        : spaceBelow >= panelHeight + offset || spaceBelow >= spaceAbove;

    // Left-aligned with the anchor by default, right-aligned when it would overflow
    const leftAligned = anchorRect.left;
    const overflowsRight = leftAligned + panelWidth > viewportWidth - margin;

    left = overflowsRight ? anchorRect.right - panelWidth : leftAligned;
    top = openBelow
      ? anchorRect.bottom + offset
      : anchorRect.top - panelHeight - offset;
  }

  // Keep the panel within the viewport edges
  if (left + panelWidth > viewportWidth - margin) {
    left = viewportWidth - panelWidth - margin;
  }
  if (left < margin) {
    left = margin;
  }
  if (top + panelHeight > viewportHeight - margin) {
    top = viewportHeight - panelHeight - margin;
  }
  if (top < margin) {
    top = margin;
  }

  return { top, left };
}

export default function useAnchorPosition(
  open: boolean,
  anchorEl: RefObject<HTMLElement | null>,
  placement: Placement = "bottom",
) {
  const panelRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState<CSSProperties>({});

  useLayoutEffect(() => {
    if (!open) {
      return;
    }

    function updatePosition() {
      const anchor = anchorEl.current;
      const panel = panelRef.current;
      if (!anchor || !panel) {
        return;
      }

      const { top, left } = getPanelPosition(anchor, panel, placement);
      setPosition({ top, left });
    }

    updatePosition();

    window.addEventListener("resize", updatePosition);
    window.addEventListener("scroll", updatePosition, true);

    return () => {
      window.removeEventListener("resize", updatePosition);
      window.removeEventListener("scroll", updatePosition, true);
    };
  }, [open, anchorEl, placement]);

  return { panelRef, position };
}
