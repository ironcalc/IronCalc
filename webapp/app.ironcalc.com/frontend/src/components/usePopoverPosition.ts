import { type CSSProperties, useLayoutEffect, useRef, useState } from "react";

function getPopoverPosition(trigger: HTMLElement, popover: HTMLElement) {
  const triggerRect = trigger.getBoundingClientRect();
  const popoverWidth = popover.offsetWidth;
  const popoverHeight = popover.offsetHeight;
  const viewportWidth = window.innerWidth;

  const offset = 6;
  const margin = 8;

  let left = triggerRect.left + triggerRect.width / 2 - popoverWidth / 2;

  let top = triggerRect.top - popoverHeight - offset;
  if (top < margin) {
    top = triggerRect.bottom + offset;
  }

  if (left + popoverWidth > viewportWidth - margin) {
    left = viewportWidth - popoverWidth - margin;
  }
  if (left < margin) {
    left = margin;
  }

  return { top, left };
}

export function usePopoverPosition(visible: boolean) {
  const triggerRef = useRef<HTMLSpanElement>(null);
  const popoverRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState<CSSProperties>({});

  useLayoutEffect(() => {
    if (!visible) {
      return;
    }

    function updatePosition() {
      const trigger = triggerRef.current;
      const popover = popoverRef.current;
      if (!trigger || !popover) {
        return;
      }

      const { top, left } = getPopoverPosition(trigger, popover);
      setPosition({ top, left });
    }

    updatePosition();

    window.addEventListener("resize", updatePosition);
    window.addEventListener("scroll", updatePosition, true);

    return () => {
      window.removeEventListener("resize", updatePosition);
      window.removeEventListener("scroll", updatePosition, true);
    };
  }, [visible]);

  return { triggerRef, popoverRef, position };
}
