import { type CSSProperties, useLayoutEffect, useRef, useState } from "react";

function getTooltipPosition(trigger: HTMLElement, tooltip: HTMLElement) {
  const triggerRect = trigger.getBoundingClientRect();
  const tooltipWidth = tooltip.offsetWidth;
  const tooltipHeight = tooltip.offsetHeight;
  const viewportWidth = window.innerWidth;

  const offset = 6; // Distance between trigger and tooltip
  const margin = 8; // Safety margin from viewport edges

  // Centered horizontally over the trigger, falls back to clamped position
  let left = triggerRect.left + triggerRect.width / 2 - tooltipWidth / 2;

  // Above by default, falls back to below when there's not enough space
  let top = triggerRect.top - tooltipHeight - offset;
  if (top < margin) {
    top = triggerRect.bottom + offset;
  }

  if (left + tooltipWidth > viewportWidth - margin) {
    left = viewportWidth - tooltipWidth - margin;
  }
  if (left < margin) {
    left = margin;
  }

  return { top, left };
}

export function useTooltipPosition(visible: boolean) {
  const triggerRef = useRef<HTMLSpanElement>(null);
  const tooltipRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState<CSSProperties>({});

  useLayoutEffect(() => {
    if (!visible) {
      return;
    }

    function updatePosition() {
      const trigger = triggerRef.current;
      const tooltip = tooltipRef.current;
      if (!trigger || !tooltip) {
        return;
      }

      const { top, left } = getTooltipPosition(trigger, tooltip);
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

  return { triggerRef, tooltipRef, position };
}
