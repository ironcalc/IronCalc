import {
  forwardRef,
  type InputHTMLAttributes,
  type ReactNode,
  useId,
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
      ...rest
    },
    ref,
  ) {
    const autoId = useId();
    const id = idProp ?? autoId;
    const helperId = `${id}-helper`;

    const controlClassName = [
      "ic-input-control",
      `${size}`,
      error && "is-error",
      disabled && "is-disabled",
      startAdornment && "has-start",
      endAdornment && "has-end",
    ]
      .filter(Boolean)
      .join(" ");

    return (
      <div
        className={["ic-input", className].filter(Boolean).join(" ")}
        style={style}
      >
        {label && (
          <label htmlFor={id} data-required={required || undefined}>
            {label}
          </label>
        )}

        <div className={controlClassName}>
          {startAdornment && <span>{startAdornment}</span>}

          <input
            ref={ref}
            id={id}
            disabled={disabled}
            required={required}
            aria-invalid={error || undefined}
            aria-describedby={helperText ? helperId : undefined}
            onKeyDown={(e) => e.stopPropagation()}
            onClick={(e) => e.stopPropagation()}
            {...rest}
          />

          {endAdornment && <span>{endAdornment}</span>}
        </div>

        {helperText && <p id={helperId}>{helperText}</p>}
      </div>
    );
  },
);

Input.displayName = "Input";
