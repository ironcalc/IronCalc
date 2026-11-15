import {
  forwardRef,
  type InputHTMLAttributes,
  type ReactNode,
  useId,
} from "react";

import "./radio.css";

/**
 * This is a reusable radio button with a label on its right and an optional
 * description under the label.
 * States: default, hover, focused, disabled.
 *
 * Radios that share a `name` form a group: checking one unchecks the others.
 */

/** Extends native `<input type="radio">` props.
 * Defaults: `disabled` false.
 * Optional: `description`.
 * `onChange` is called when this radio becomes checked.
 */

export interface RadioProperties
  extends Omit<
    InputHTMLAttributes<HTMLInputElement>,
    "type" | "onChange" | "size"
  > {
  checked: boolean;
  onChange: () => void;
  label: ReactNode;
  description?: ReactNode;
}

export const Radio = forwardRef<HTMLInputElement, RadioProperties>(
  function Radio(
    {
      checked,
      onChange,
      label,
      description,
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
    const descriptionId = `${id}-description`;
    const radioClassName = ["ic-radio", disabled && "disabled", className]
      .filter(Boolean)
      .join(" ");
    return (
      <label className={radioClassName} style={style} htmlFor={id}>
        <input
          ref={ref}
          id={id}
          type="radio"
          checked={checked}
          disabled={disabled}
          onChange={(event) => {
            if (event.target.checked) {
              onChange();
            }
          }}
          aria-describedby={description ? descriptionId : undefined}
          className="ic-radio-input"
          {...rest}
        />
        <span className="ic-radio-circle" aria-hidden="true" />
        <span className="ic-radio-text">
          <span className="ic-radio-label">{label}</span>
          {description && (
            <span id={descriptionId} className="ic-radio-description">
              {description}
            </span>
          )}
        </span>
      </label>
    );
  },
);

Radio.displayName = "Radio";
