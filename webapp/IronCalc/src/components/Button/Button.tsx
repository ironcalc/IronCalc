import {
  type ButtonHTMLAttributes,
  type CSSProperties,
  forwardRef,
  type ReactNode,
} from "react";

import "./button.css";

/**
 * This is a reusable text Button with optional start/end icons.
 * Variants: primary, secondary, outline, ghost (no border), destructive.
 * Sizes: xs, sm, md.
 */

export type ButtonVariant =
  | "primary"
  | "secondary"
  | "outline"
  | "ghost"
  | "destructive";
export type ButtonSize = "xs" | "sm" | "md";

export interface ButtonStyles {
  variant: ButtonVariant;
  size: ButtonSize;
  pressed: boolean;
  disabled: boolean;
  hovered: boolean;
}

export const iconWrapperStyle: CSSProperties = {
  width: 16,
  height: 16,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
};

/** Extends native `<button>` props.
 * Defaults: `variant` "primary", `size` "md", `pressed` false.
 * Optional: `startIcon`, `endIcon`.
 */

export interface ButtonProperties
  extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  pressed?: boolean;
  startIcon?: ReactNode;
  endIcon?: ReactNode;
}


export const Button = forwardRef<HTMLButtonElement, ButtonProperties>(
  function Button(
    {
      variant = "primary",
      size = "md",
      pressed = false,
      disabled = false,
      children,
      startIcon,
      endIcon,
      style,
      ...rest
    },
    ref,
  ) {
  const buttonClassName = [
      "ic-button",
      `ic-button--${variant}`,
      `ic-button--${size}`,
    ].filter(Boolean)
  .join(" ");
    return (
      <button
        ref={ref}
        disabled={disabled}
        aria-pressed={pressed}
        data-pressed={pressed ? "true" : undefined}
        className={buttonClassName}
        style={style}
        {...rest}
      >
        {startIcon && <span className="ic-button__icon">{startIcon}</span>}
        {children}
        {endIcon && <span className="ic-button__icon">{endIcon}</span>}
      </button>
    );
  },
);

Button.displayName = "Button";
