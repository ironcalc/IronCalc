import {
  type CSSProperties,
  type RefObject,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

function getMenuPosition(trigger: HTMLElement, menu: HTMLElement) {
  const triggerRect = trigger.getBoundingClientRect();
  const menuWidth = menu.offsetWidth;
  const menuHeight = menu.offsetHeight;
  const viewportWidth = window.innerWidth;
  const viewportHeight = window.innerHeight;

  const offset = 4; // Distance between trigger and menu
  const margin = 8; // Safety margin from viewport edges

  // Below by default, falls back to above when there's more space there
  const spaceBelow = viewportHeight - triggerRect.bottom - margin;
  const spaceAbove = triggerRect.top - margin;
  const openBelow =
    spaceBelow >= menuHeight + offset || spaceBelow >= spaceAbove;

  // Left-aligned with the trigger by default, right-aligned when it would overflow
  const leftAligned = triggerRect.left;
  const overflowsRight = leftAligned + menuWidth > viewportWidth - margin;

  let left = overflowsRight ? triggerRect.right - menuWidth : leftAligned;
  let top = openBelow
    ? triggerRect.bottom + offset
    : triggerRect.top - menuHeight - offset;

  // Keep the menu within the viewport edges
  if (left + menuWidth > viewportWidth - margin) {
    left = viewportWidth - menuWidth - margin;
  }
  if (left < margin) {
    left = margin;
  }
  if (top + menuHeight > viewportHeight - margin) {
    top = viewportHeight - menuHeight - margin;
  }
  if (top < margin) {
    top = margin;
  }

  return { top, left };
}

/**
 * Positions a popup (menu, formula helper…) next to an anchor element,
 * flipping above / right-aligning and clamping so it stays in the viewport.
 *
 * The anchor is either the returned `triggerRef` (attach it to the trigger
 * element) or, when the caller already owns a ref to the anchor (e.g. the
 * editor textarea), the optional `anchorRef` argument.
 */
export function useMenuPosition(
  open: boolean,
  anchorRef?: RefObject<HTMLElement | null>,
) {
  const triggerRef = useRef<HTMLElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState<CSSProperties>({});

  useLayoutEffect(() => {
    if (!open) {
      return;
    }
    const anchor = anchorRef ?? triggerRef;

    function updatePosition() {
      const trigger = anchor.current;
      const menu = menuRef.current;
      if (!trigger || !menu) {
        return;
      }

      const { top, left } = getMenuPosition(trigger, menu);
      setPosition({ top, left });
    }

    updatePosition();

    // The anchor and the popup can both change size while open (e.g. the
    // editor textarea grows while typing, the formula helper switches cards
    // or collapses), so watch both in addition to viewport resize/scroll.
    const observer = new ResizeObserver(updatePosition);
    if (anchor.current) {
      observer.observe(anchor.current);
    }
    if (menuRef.current) {
      observer.observe(menuRef.current);
    }
    window.addEventListener("resize", updatePosition);
    window.addEventListener("scroll", updatePosition, true);

    return () => {
      observer.disconnect();
      window.removeEventListener("resize", updatePosition);
      window.removeEventListener("scroll", updatePosition, true);
    };
  }, [open, anchorRef]);

  return { triggerRef, menuRef, position };
}
