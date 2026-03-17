import { useTheme } from "@mui/material";
import {
  type ButtonHTMLAttributes,
  forwardRef,
  type ReactNode,
  useState,
} from "react";
import {
  type ButtonSize,
  type ButtonVariant,
  getButtonStyles,
  iconWrapperStyle,
} from "./Button";

export type { ButtonSize, ButtonVariant };

/**
 * Icon-only button. Same variants and sizes as Button.
 * Use it for toolbar actions, to close drawers and modals, etc.
 * Always provide aria-label for accessibility.
 */
export interface IconButtonProperties
  extends ButtonHTMLAttributes<HTMLButtonElement> {
  icon: ReactNode;
  variant?: ButtonVariant;
  size?: ButtonSize;
  pressed?: boolean;
}

export const IconButton = forwardRef<HTMLButtonElement, IconButtonProperties>(
  function IconButton(
    {
      icon,
      variant = "primary",
      size = "md",
      pressed = false,
      style,
      disabled = false,
      onMouseEnter,
      onMouseLeave,
      ...rest
    },
    ref,
  ) {
    const theme = useTheme();
    const [hovered, setHovered] = useState(false);
    const computedStyles = getButtonStyles(
      theme,
      variant,
      size,
      true,
      pressed,
      disabled,
      hovered,
    );

    return (
      <button
        ref={ref}
        disabled={disabled}
        aria-pressed={pressed}
        style={{ ...computedStyles, ...style }}
        onMouseEnter={(e) => {
          setHovered(true);
          onMouseEnter?.(e);
        }}
        onMouseLeave={(e) => {
          setHovered(false);
          onMouseLeave?.(e);
        }}
        {...rest}
      >
        <span style={iconWrapperStyle}>{icon}</span>
      </button>
    );
  },
);

IconButton.displayName = "IconButton";
