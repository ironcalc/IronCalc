import { type ReactElement, useId, useState } from "react";
import { createPortal } from "react-dom";

import "./tooltip.css";
import { useTooltipPosition } from "./useTooltipPosition";

/**
 * Reusable Tooltip component
 * Placed on top by default, fallbacks to bottom when there's no space
 */

export interface TooltipProperties {
  title: string;
  children: ReactElement;
}

export function Tooltip({ title, children }: TooltipProperties) {
  const tooltipId = useId();
  const [visible, setVisible] = useState(false);
  const { triggerRef, tooltipRef, position } = useTooltipPosition(visible);

  return (
    <>
      <span
        ref={triggerRef}
        role="none"
        className="ic-tooltip-trigger"
        aria-describedby={visible ? tooltipId : undefined}
        onMouseEnter={() => setVisible(true)}
        onMouseLeave={() => setVisible(false)}
        onFocus={() => setVisible(true)}
        onBlur={() => setVisible(false)}
      >
        {children}
      </span>

      {createPortal(
        <div
          ref={tooltipRef}
          id={tooltipId}
          role="tooltip"
          className="ic-tooltip"
          data-visible={visible}
          style={position}
        >
          {title}
        </div>,
        document.body,
      )}
    </>
  );
}

Tooltip.displayName = "Tooltip";
