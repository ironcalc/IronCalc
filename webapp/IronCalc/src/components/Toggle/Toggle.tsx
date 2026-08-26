import {
  forwardRef,
  type InputHTMLAttributes,
  type ReactNode,
  useId,
} from "react";

import "./toggle.css";

/**
 * This is a reusable toggle switch with an optional label on its left.
 * States: default, hover, focused, disabled.
 */

/** Extends native `<input type="checkbox">` props.
 * Defaults: `disabled` false.
 * Optional: `label`.
 * `onChange` receives the new checked value, not the event.
 */

export interface ToggleProperties
  extends Omit<
    InputHTMLAttributes<HTMLInputElement>,
    "type" | "onChange" | "size"
  > {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: ReactNode;
}

export const Toggle = forwardRef<HTMLInputElement, ToggleProperties>(
  function Toggle(
    {
      checked,
      onChange,
      label,
      disabled = false,
      id: idProperty,
      style,
      className,
      ...rest
    },
    ref,
  ) {
    const autoId = useId();
    const id = idProperty ?? autoId;
    const toggleClassName = ["ic-toggle", disabled && "disabled", className]
      .filter(Boolean)
      .join(" ");
    return (
      <label className={toggleClassName} style={style} htmlFor={id}>
        {label && <span className="ic-toggle-label">{label}</span>}
        <input
          ref={ref}
          id={id}
          type="checkbox"
          role="switch"
          aria-checked={checked}
          checked={checked}
          disabled={disabled}
          onChange={(event) => onChange(event.target.checked)}
          className="ic-toggle-input"
          {...rest}
        />
        <span className="ic-toggle-track" aria-hidden="true">
          <span className="ic-toggle-thumb" />
        </span>
      </label>
    );
  },
);

Toggle.displayName = "Toggle";
