import { type CSSProperties, useLayoutEffect, useRef, useState } from "react";

function getMenuPosition(trigger: HTMLElement, menu: HTMLElement) {
  const triggerRect = trigger.getBoundingClientRect();
  const menuWidth = menu.offsetWidth;
  const menuHeight = menu.offsetHeight;
  const viewportWidth = window.innerWidth;
  const viewportHeight = window.innerHeight;

  const offset = 4;
  const margin = 8;

  const spaceBelow = viewportHeight - triggerRect.bottom - margin;
  const spaceAbove = triggerRect.top - margin;
  const openBelow =
    spaceBelow >= menuHeight + offset || spaceBelow >= spaceAbove;

  const leftAligned = triggerRect.left;
  const overflowsRight = leftAligned + menuWidth > viewportWidth - margin;

  let left = overflowsRight ? triggerRect.right - menuWidth : leftAligned;
  let top = openBelow
    ? triggerRect.bottom + offset
    : triggerRect.top - menuHeight - offset;

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

export function useMenuPosition(open: boolean) {
  const triggerRef = useRef<HTMLElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState<CSSProperties>({});

  useLayoutEffect(() => {
    if (!open) {
      return;
    }

    function updatePosition() {
      const trigger = triggerRef.current;
      const menu = menuRef.current;
      if (!trigger || !menu) {
        return;
      }

      const { top, left } = getMenuPosition(trigger, menu);
      setPosition({ top, left });
    }

    updatePosition();

    window.addEventListener("resize", updatePosition);
    window.addEventListener("scroll", updatePosition, true);

    return () => {
      window.removeEventListener("resize", updatePosition);
      window.removeEventListener("scroll", updatePosition, true);
    };
  }, [open]);

  return { triggerRef, menuRef, position };
}
