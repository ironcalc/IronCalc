import {
  forwardRef,
  type InputHTMLAttributes,
  type ReactNode,
  useId,
  useState,
} from "react";

import "./input.css";

/**
 * Reusable text Input with optional start/end adornments, label, and helper text.
 * Sizes: sm, md
 * States: default, hover, focused, error, disabled.
 */

export type InputSize = "sm" | "md";

/** Extends native `<input>` props.
 * Defaults: `size` "md".
 * Optional: `label`, `helperText`, `error`, `startAdornment`, `endAdornment`.
 */

export interface InputProperties
  extends Omit<InputHTMLAttributes<HTMLInputElement>, "size"> {
  size?: InputSize;
  label?: ReactNode;
  helperText?: ReactNode;
  error?: boolean;
  startAdornment?: ReactNode;
  endAdornment?: ReactNode;
}

export const Input = forwardRef<HTMLInputElement, InputProperties>(
  function Input(
    {
      size = "md",
      label,
      helperText,
      error = false,
      disabled = false,
      startAdornment,
      endAdornment,
      required,
      id: idProp,
      className,
      style,
      onFocus,
      onBlur,
      ...rest
    },
    ref,
  ) {
    const autoId = useId();
    const id = idProp ?? autoId;
    const helperId = `${id}-helper`;

    const [focused, setFocused] = useState(false);

    const wrapperClassName = [
      "ic-input-wrapper",
      `ic-input-wrapper--${size}`,
      focused && "ic-input-wrapper--focused",
      error && "ic-input-wrapper--error",
      disabled && "ic-input-wrapper--disabled",
      startAdornment && "ic-input-wrapper--has-start",
      endAdornment && "ic-input-wrapper--has-end",
    ]
      .filter(Boolean)
      .join(" ");

    return (
      <div
        className={["ic-input-root", className].filter(Boolean).join(" ")}
        style={style}
      >
        {label && (
          <label
            htmlFor={id}
            className={[
              "ic-input-label",
              disabled && "ic-input-label--disabled",
              required && "ic-input-label--required",
            ]
              .filter(Boolean)
              .join(" ")}
          >
            {label}
          </label>
        )}

        <div className={wrapperClassName}>
          {startAdornment && (
            <span
              className={[
                "ic-input-adornment",
                "ic-input-adornment--start",
                disabled && "ic-input-adornment--disabled",
              ]
                .filter(Boolean)
                .join(" ")}
            >
              {startAdornment}
            </span>
          )}

          <input
            ref={ref}
            id={id}
            disabled={disabled}
            required={required}
            aria-invalid={error || undefined}
            aria-describedby={helperText ? helperId : undefined}
            className="ic-input-field"
            onFocus={(e) => {
              setFocused(true);
              onFocus?.(e);
            }}
            onBlur={(e) => {
              setFocused(false);
              onBlur?.(e);
            }}
            {...rest}
          />

          {endAdornment && (
            <span
              className={[
                "ic-input-adornment",
                "ic-input-adornment--end",
                disabled && "ic-input-adornment--disabled",
              ]
                .filter(Boolean)
                .join(" ")}
            >
              {endAdornment}
            </span>
          )}
        </div>

        {helperText && (
          <p
            id={helperId}
            className={[
              "ic-input-helper-text",
              error && "ic-input-helper-text--error",
              disabled && "ic-input-helper-text--disabled",
            ]
              .filter(Boolean)
              .join(" ")}
          >
            {helperText}
          </p>
        )}
      </div>
    );
  },
);

Input.displayName = "Input";
