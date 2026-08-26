import {
  forwardRef,
  type InputHTMLAttributes,
  type ReactNode,
  useId,
} from "react";

import "./checkbox.css";

/**
 * This is a reusable checkbox with an optional label on its right.
 * States: default, hover, focused, disabled.
 */

/** Extends native `<input type="checkbox">` props.
 * Defaults: `disabled` false.
 * Optional: `label`.
 * `onChange` receives the new checked value, not the event.
 */

export interface CheckboxProperties
  extends Omit<
    InputHTMLAttributes<HTMLInputElement>,
    "type" | "onChange" | "size"
  > {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: ReactNode;
}

export const Checkbox = forwardRef<HTMLInputElement, CheckboxProperties>(
  function Checkbox(
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
    const checkboxClassName = ["ic-checkbox", disabled && "disabled", className]
      .filter(Boolean)
      .join(" ");
    return (
      <label className={checkboxClassName} style={style} htmlFor={id}>
        <input
          ref={ref}
          id={id}
          type="checkbox"
          checked={checked}
          disabled={disabled}
          onChange={(event) => onChange(event.target.checked)}
          className="ic-checkbox-input"
          {...rest}
        />
        <span className="ic-checkbox-box" aria-hidden="true" />
        {label && <span className="ic-checkbox-label">{label}</span>}
      </label>
    );
  },
);

Checkbox.displayName = "Checkbox";
