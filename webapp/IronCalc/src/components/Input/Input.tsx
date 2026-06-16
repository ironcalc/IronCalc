import { Minus, Plus } from "lucide-react";
import {
  forwardRef,
  type InputHTMLAttributes,
  type ReactNode,
  useId,
  useState,
} from "react";
import { IconButton } from "../Button/IconButton";

import "./input.css";

/**
 * Reusable text Input with optional start/end adornments, label, and helper text.
 * Sizes: sm, md
 * States: default, hover, focused, error, disabled.
 */

export type InputSize = "sm" | "md";

/** Extends native `<input>` props.
 * Defaults: `size` "md".
 * Optional: `label`, `helperText`, `error`, `startAdornment`, `endAdornment`, `numberInput`.
 */

export interface InputProperties
  extends Omit<InputHTMLAttributes<HTMLInputElement>, "size"> {
  size?: InputSize;
  label?: ReactNode;
  helperText?: ReactNode;
  error?: boolean;
  startAdornment?: ReactNode;
  endAdornment?: ReactNode;
  numberInput?: boolean;
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
      numberInput = false,
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
    const [editMode, setEditMode] = useState(false);

    const handleStep = (
      direction: "up" | "down",
      event: React.MouseEvent,
    ): void => {
      const current = Number(rest.value) || 0;
      const baseStep =
        rest.step === undefined || rest.step === "any"
          ? 1
          : Number(rest.step) || 1;
      const stepSize = event.shiftKey ? baseStep * 10 : baseStep;
      const minVal = rest.min !== undefined ? Number(rest.min) : -Infinity;
      const maxVal = rest.max !== undefined ? Number(rest.max) : Infinity;
      const next = direction === "up" ? current + stepSize : current - stepSize;
      const clamped = Math.max(minVal, Math.min(maxVal, next));
      rest.onChange?.({
        target: { value: String(clamped) },
      } as React.ChangeEvent<HTMLInputElement>);
    };

    const controlClassName = [
      "ic-input-control",
      `${size}`,
      error && "is-error",
      disabled && "is-disabled",
      !numberInput && startAdornment && "has-start",
      !numberInput && endAdornment && "has-end",
      numberInput && "is-number-input",
    ]
      .filter(Boolean)
      .join(" ");

    const inputControl = (
      <div className={controlClassName}>
        {numberInput ? (
          <>
            <IconButton
              icon={<Minus />}
              aria-label="Decrease"
              variant="ghost"
              size="xs"
              disabled={disabled}
              className="ic-input-number-btn"
              onClick={(e) => handleStep("down", e)}
            />
            {editMode ? (
              <input
                ref={ref}
                id={id}
                // biome-ignore lint/a11y/noAutofocus: user explicitly clicked to enter edit mode
                autoFocus
                disabled={disabled}
                required={required}
                aria-invalid={error || undefined}
                aria-describedby={helperText ? helperId : undefined}
                onBlur={() => setEditMode(false)}
                onKeyDown={(e) => {
                  e.stopPropagation();
                  if (e.key === "Enter" || e.key === "Escape") {
                    setEditMode(false);
                  }
                }}
                onClick={(e) => e.stopPropagation()}
                onPaste={(e) => e.stopPropagation()}
                onCopy={(e) => e.stopPropagation()}
                onCut={(e) => e.stopPropagation()}
                onFocus={(e) => e.target.select()}
                spellCheck={false}
                {...rest}
              />
            ) : (
              <button
                type="button"
                className="ic-input-number-display"
                disabled={disabled}
                onClick={() => setEditMode(true)}
              >
                <span>{rest.value}</span>
                {endAdornment && (
                  <span className="ic-input-number-unit">{endAdornment}</span>
                )}
              </button>
            )}
            <IconButton
              icon={<Plus />}
              aria-label="Increase"
              variant="ghost"
              size="xs"
              disabled={disabled}
              className="ic-input-number-btn"
              onClick={(e) => handleStep("up", e)}
            />
          </>
        ) : (
          <>
            {startAdornment && <span>{startAdornment}</span>}
            <input
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
              onFocus={(e) => e.target.select()}
              spellCheck={false}
              {...rest}
            />
            {endAdornment && <span>{endAdornment}</span>}
          </>
        )}
      </div>
    );

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

        {inputControl}

        {helperText && <p id={helperId}>{helperText}</p>}
      </div>
    );
  },
);

Input.displayName = "Input";
