import type { HTMLAttributes, ReactNode } from "react";
import { Button, type ButtonSize } from "../Button/Button";
import { IconButton } from "../Button/IconButton";

import "./toggle-button.css";

/**
 * A group of mutually exclusive buttons where exactly one option is selected.
 * Every option is a "ghost" Button; the selected one is the pressed Button.
 * Sizes: xs, sm, md (same as Button).
 */

export interface ToggleButtonOption<T extends string> {
  value: T;
  /** Text of the button. Omit it to get an icon-only button. */
  label?: ReactNode;
  icon?: ReactNode;
  /** Required when there is no `label`. */
  "aria-label"?: string;
  disabled?: boolean;
}

/** Extends native `<div>` props.
 * Defaults: `size` "sm", `fullWidth` false.
 * `onChange` receives the new value, not the event.
 */

export interface ToggleButtonProperties<T extends string>
  extends Omit<HTMLAttributes<HTMLDivElement>, "onChange"> {
  options: ToggleButtonOption<T>[];
  value: T;
  onChange: (value: T) => void;
  size?: ButtonSize;
  /** Stretch the group and share its width equally between the options. */
  fullWidth?: boolean;
  disabled?: boolean;
}

// Not a forwardRef like the other components: a generic forwardRef needs a cast
// that loses the inference on `value`/`onChange`, and nobody needs the ref.
export function ToggleButton<T extends string>({
  options,
  value,
  onChange,
  size = "sm",
  fullWidth = false,
  disabled = false,
  className,
  style,
  ...rest
}: ToggleButtonProperties<T>) {
  const groupClassName = [
    "ic-toggle-button",
    fullWidth && "ic-toggle-button--full-width",
    className,
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <div className={groupClassName} style={style} {...rest}>
      {options.map((option) => {
        const selected = option.value === value;
        const properties = {
          size,
          variant: "ghost",
          pressed: selected,
          disabled: disabled || option.disabled,
          onClick: () => onChange(option.value),
        } as const;

        return option.label === undefined ? (
          <IconButton
            key={option.value}
            icon={option.icon}
            aria-label={option["aria-label"] ?? option.value}
            {...properties}
          />
        ) : (
          <Button
            key={option.value}
            startIcon={option.icon}
            aria-label={option["aria-label"]}
            {...properties}
          >
            {option.label}
          </Button>
        );
      })}
    </div>
  );
}
