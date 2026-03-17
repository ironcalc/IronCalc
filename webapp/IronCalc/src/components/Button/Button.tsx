import { useTheme } from "@mui/material";
import { alpha, type Theme } from "@mui/material/styles";
import {
  type ButtonHTMLAttributes,
  type CSSProperties,
  forwardRef,
  type ReactNode,
  useState,
} from "react";

/**
 * This is a reusable text Button with optional start/end icons.
 * Variants: primary, secondary, outline, ghost (no border), destructive.
 * Sizes: xs, sm, md. Styled with MUI theme.
 */

export type ButtonVariant =
  | "primary"
  | "secondary"
  | "outline"
  | "ghost"
  | "destructive";
export type ButtonSize = "xs" | "sm" | "md";

const sizeStyles: Record<ButtonSize, CSSProperties> = {
  xs: { height: 24, lineHeight: "24px" },
  sm: { height: 28, lineHeight: "28px" },
  md: { height: 32, lineHeight: "32px" },
};

export function getButtonStyles(
  theme: Theme,
  variant: ButtonVariant,
  size: ButtonSize,
  iconOnly: boolean,
  pressed: boolean,
  disabled: boolean,
  hovered: boolean,
): CSSProperties {
  const { height, lineHeight } = sizeStyles[size];

  const base: CSSProperties = {
    cursor: disabled ? "not-allowed" : "pointer",
    position: "relative",
    overflow: "hidden",
    padding: iconOnly ? 0 : "0 10px",
    borderRadius: 6,
    display: "inline-flex",
    alignItems: "center",
    justifyContent: "center",
    gap: iconOnly ? 0 : 8,
    fontFamily: theme.typography.fontFamily,
    fontSize: theme.typography.fontSize,
    fontWeight: 500,
    textDecoration: "none",
    boxSizing: "border-box",
    border: `1px solid ${alpha(theme.palette.common.black, 0.04)}`,
    boxShadow: pressed
      ? `inset 0 1px 1px ${alpha(theme.palette.common.black, 0.12)}`
      : `${alpha(theme.palette.common.black, 0.04)} 0px 1px 2px`,
    height,
    lineHeight,
    ...(iconOnly ? { minWidth: height, width: height } : {}),
  };

  if (disabled) {
    return {
      ...base,
      backgroundColor: theme.palette.grey[200],
      color: theme.palette.action.disabled,
      borderColor: theme.palette.grey[300],
      boxShadow: "none",
    };
  }

  const showHover = hovered && !disabled;

  const variantStyles: Record<ButtonVariant, CSSProperties> = {
    primary: {
      backgroundColor: showHover
        ? theme.palette.primary.dark
        : pressed
          ? theme.palette.primary.dark
          : theme.palette.primary.main,
      color: theme.palette.primary.contrastText,
    },
    secondary: {
      backgroundColor: showHover
        ? theme.palette.grey[300]
        : pressed
          ? theme.palette.grey[300]
          : theme.palette.grey[200],
      color: theme.palette.common.black,
    },
    outline: {
      backgroundColor: showHover
        ? theme.palette.grey[100]
        : pressed
          ? theme.palette.grey[100]
          : "transparent",
      color: theme.palette.common.black,
      borderColor: theme.palette.grey[300],
      boxShadow: "none",
    },
    ghost: {
      backgroundColor: showHover
        ? theme.palette.grey[100]
        : pressed
          ? theme.palette.grey[200]
          : "transparent",
      color: theme.palette.common.black,
      border: "none",
      boxShadow: "none",
    },
    destructive: {
      backgroundColor: showHover
        ? theme.palette.error.dark
        : pressed
          ? theme.palette.error.dark
          : theme.palette.error.main,
      color: theme.palette.error.contrastText,
    },
  };

  return { ...base, ...variantStyles[variant] };
}

export const iconWrapperStyle: CSSProperties = {
  width: 16,
  height: 16,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
};

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
      children,
      startIcon,
      endIcon,
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
      false, // text button, never icon-only
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
        {startIcon && <span style={iconWrapperStyle}>{startIcon}</span>}
        {children}
        {endIcon && <span style={iconWrapperStyle}>{endIcon}</span>}
      </button>
    );
  },
);

Button.displayName = "Button";
