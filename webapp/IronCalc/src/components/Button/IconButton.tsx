import { type ButtonHTMLAttributes, forwardRef, type ReactNode } from "react";
import {
  type ButtonSize,
  type ButtonVariant,
  iconWrapperStyle,
} from "./Button";

export type { ButtonSize, ButtonVariant };

/**
 * Icon-only button. Same variants and sizes as Button.
 * Use it for toolbar actions, to close drawers and modals, etc.
 * Defaults: `variant` "ghost", `size` "sm", `pressed` false.
 */
export interface IconButtonProperties
  extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "aria-label"> {
  icon: ReactNode;
  "aria-label": string;
  variant?: ButtonVariant;
  size?: ButtonSize;
  pressed?: boolean;
}

export const IconButton = forwardRef<HTMLButtonElement, IconButtonProperties>(
  function IconButton(
    {
      icon,
      variant = "ghost",
      size = "sm",
      pressed = false,
      disabled = false,
      style,
      ...rest
    },
    ref,
  ) {
    const buttonClassName = [
      "ic-button",
      "ic-button--icon-only",
      `ic-button--${variant}`,
      `ic-button--${size}`,
    ].join(" ");

    return (
      <button
        ref={ref}
        className={buttonClassName}
        disabled={disabled}
        aria-pressed={pressed}
        style={style}
        {...rest}
      >
        <span style={iconWrapperStyle}>{icon}</span>
      </button>
    );
  },
);

IconButton.displayName = "IconButton";
