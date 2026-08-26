import {
  forwardRef,
  type InputHTMLAttributes,
  type ReactNode,
  useId,
} from "react";

import "./switch.css";

/**
 * This is a reusable switch with an optional label on its left.
 * States: default, hover, focused, disabled.
 */

/** Extends native `<input type="checkbox">` props.
 * Defaults: `disabled` false.
 * Optional: `label`.
 * `onChange` receives the new checked value, not the event.
 */

export interface SwitchProperties
  extends Omit<
    InputHTMLAttributes<HTMLInputElement>,
    "type" | "onChange" | "size"
  > {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: ReactNode;
}

export const Switch = forwardRef<HTMLInputElement, SwitchProperties>(
  function Switch(
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
    const switchClassName = ["ic-switch", disabled && "disabled", className]
      .filter(Boolean)
      .join(" ");
    return (
      <label className={switchClassName} style={style} htmlFor={id}>
        {label && <span className="ic-switch-label">{label}</span>}
        <input
          ref={ref}
          id={id}
          type="checkbox"
          role="switch"
          aria-checked={checked}
          checked={checked}
          disabled={disabled}
          onChange={(event) => onChange(event.target.checked)}
          className="ic-switch-input"
          {...rest}
        />
        <span className="ic-switch-track" aria-hidden="true">
          <span className="ic-switch-thumb" />
        </span>
      </label>
    );
  },
);

Switch.displayName = "Switch";
