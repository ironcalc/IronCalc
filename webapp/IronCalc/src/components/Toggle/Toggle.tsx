import { type ReactNode, useId } from "react";

import "./toggle.css";

/**
 * Reusable toggle switch with an optional label shown on its left.
 * States: default, hover, focused, disabled.
 */

export interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: ReactNode;
  disabled?: boolean;
  id?: string;
  className?: string;
}

export function Toggle({
  checked,
  onChange,
  label,
  disabled = false,
  id,
  className,
}: ToggleProps) {
  const autoId = useId();
  const toggleId = id ?? autoId;

  return (
    <label
      className={["ic-toggle", disabled && "disabled", className]
        .filter(Boolean)
        .join(" ")}
      htmlFor={toggleId}
    >
      {label && <span className="ic-toggle-label">{label}</span>}
      <input
        id={toggleId}
        type="checkbox"
        role="switch"
        aria-checked={checked}
        className="ic-toggle-input"
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.target.checked)}
      />
      <span className="ic-toggle-track" aria-hidden="true">
        <span className="ic-toggle-thumb" />
      </span>
    </label>
  );
}

Toggle.displayName = "Toggle";
