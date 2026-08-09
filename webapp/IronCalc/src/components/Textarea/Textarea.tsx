import {
  forwardRef,
  type ReactNode,
  type TextareaHTMLAttributes,
  useId,
} from "react";

import "./textarea.css";

/**
 * Reusable multiline Textarea with optional label and helper text.
 * Sizes: sm, md
 * States: default, hover, focused, error, disabled.
 */

export type TextareaSize = "sm" | "md";

/** Extends native `<textarea>` props.
 * Defaults: `size` "md".
 * Optional: `label`, `helperText`, `error`.
 */

export interface TextareaProperties
  extends Omit<TextareaHTMLAttributes<HTMLTextAreaElement>, "size"> {
  size?: TextareaSize;
  label?: ReactNode;
  helperText?: ReactNode;
  error?: boolean;
  endAdornment?: ReactNode;
}

export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProperties>(
  function Textarea(
    {
      size = "md",
      label,
      helperText,
      error = false,
      disabled = false,
      required,
      endAdornment,
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
      "ic-textarea-control",
      `${size}`,
      error && "is-error",
      disabled && "is-disabled",
      endAdornment && "has-end",
    ]
      .filter(Boolean)
      .join(" ");

    return (
      <div
        className={["ic-textarea", className].filter(Boolean).join(" ")}
        style={style}
      >
        {label && (
          <label htmlFor={id} data-required={required || undefined}>
            {label}
          </label>
        )}

        <div className={controlClassName}>
          <textarea
            ref={ref}
            id={id}
            disabled={disabled}
            required={required}
            aria-invalid={error || undefined}
            aria-describedby={helperText ? helperId : undefined}
            // FIXME: the stopPropagation everywhere is because of my (Nicolás Hatcher)
            // bad implementation of keyboard handling in the spreadsheet
            onKeyDown={(e) => e.stopPropagation()}
            onClick={(e) => e.stopPropagation()}
            onPaste={(e) => e.stopPropagation()}
            onCopy={(e) => e.stopPropagation()}
            onCut={(e) => e.stopPropagation()}
            spellCheck={false}
            {...rest}
          />
          {endAdornment && (
            <span className="ic-textarea-end-adornment">{endAdornment}</span>
          )}
        </div>

        {helperText && <p id={helperId}>{helperText}</p>}
      </div>
    );
  },
);

Textarea.displayName = "Textarea";
