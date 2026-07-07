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

export function useHelperPosition(
  open: boolean,
  anchorRef: RefObject<HTMLElement | null>,
) {
  const helperRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState<CSSProperties>({});

  useLayoutEffect(() => {
    if (!open) {
      return;
    }

    function updatePosition() {
      const anchor = anchorRef.current;
      const helper = helperRef.current;
      if (!anchor || !helper) {
        return;
      }

      const { top, left } = getMenuPosition(anchor, helper);
      setPosition({ top, left });
    }

    updatePosition();

    // The textarea grows while typing and the helper switches between the
    // list and detail cards (and can be collapsed), so watch both for size
    // changes in addition to viewport resize/scroll.
    const observer = new ResizeObserver(updatePosition);
    if (anchorRef.current) {
      observer.observe(anchorRef.current);
    }
    if (helperRef.current) {
      observer.observe(helperRef.current);
    }
    window.addEventListener("resize", updatePosition);
    window.addEventListener("scroll", updatePosition, true);

    return () => {
      observer.disconnect();
      window.removeEventListener("resize", updatePosition);
      window.removeEventListener("scroll", updatePosition, true);
    };
  }, [open, anchorRef]);

  return { helperRef, position };
}
