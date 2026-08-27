import type { HTMLAttributes, ReactNode } from "react";
import { Button, type ButtonSize } from "../Button/Button";
import { IconButton } from "../Button/IconButton";

import "./toggle-button.css";

/**
 * A group of buttons where the selected ones are pressed. Every option is a
 * "ghost" Button.
 * By default the options are mutually exclusive: at most one is selected, and a
 * `value` that matches no option leaves them all unpressed, which is how a
 * single option toggles on and off.
 * With `multiple`, each option toggles independently and `value` is the array of
 * the selected ones. Adjacent selected options are drawn as one run.
 * Sizes: xs, sm, md (same as Button).
 */

export interface ToggleButtonOption<T extends string> {
  value: T;
  label?: ReactNode /** Text of the button. Omit it to get an icon-only button. */;
  icon?: ReactNode;
  "aria-label"?: string /** Accessible name. Icon-only buttons fall back to `value`. */;
  disabled?: boolean;
}

/** Extends native `<div>` props.
 * Defaults: `size` "sm", `fullWidth` false, `multiple` false.
 * `onChange` receives the new value, not the event.
 */

interface ToggleButtonBaseProperties<T extends string>
  extends Omit<HTMLAttributes<HTMLDivElement>, "onChange"> {
  options: ToggleButtonOption<T>[];
  size?: ButtonSize;
  fullWidth?: boolean;
  disabled?: boolean;
}

export type ToggleButtonProperties<T extends string> =
  ToggleButtonBaseProperties<T> &
    (
      | { multiple?: false; value: T; onChange: (value: T) => void }
      | { multiple: true; value: T[]; onChange: (values: T[]) => void }
    );

export function ToggleButton<T extends string>({
  options,
  value,
  onChange,
  multiple,
  size = "sm",
  fullWidth = false,
  disabled = false,
  className,
  style,
  ...rest
}: ToggleButtonProperties<T>) {
  const selectedValues = multiple ? value : [value];

  const toggle = (optionValue: T) => {
    if (multiple) {
      onChange(
        value.includes(optionValue)
          ? value.filter((selected) => selected !== optionValue)
          : [...value, optionValue],
      );
    } else {
      onChange(optionValue);
    }
  };

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
        const properties = {
          size,
          variant: "ghost",
          pressed: selectedValues.includes(option.value),
          disabled: disabled || option.disabled,
          onClick: () => toggle(option.value),
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
