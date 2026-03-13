import { useTheme } from "@mui/material";
import type { Theme } from "@mui/material/styles";
import {
  type ButtonHTMLAttributes,
  forwardRef,
  type ReactNode,
  useState,
} from "react";

export type ButtonVariant =
  | "primary"
  | "secondary"
  | "outline"
  | "ghost"
  | "destructive";
export type ButtonSize = "xs" | "sm" | "md" | "lg";

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant: ButtonVariant;
  size: ButtonSize;
  iconOnly: boolean;
  // Toggle state: keeps the button visually pressed (e.g. Bold/Italic when formatting is on)
  pressed: boolean;
  startIcon: ReactNode;
  endIcon: ReactNode;
}

const sizeStyles: Record<ButtonSize, React.CSSProperties> = {
  xs: { height: 24, lineHeight: "24px" },
  sm: { height: 28, lineHeight: "28px" },
  md: { height: 32, lineHeight: "32px" },
  lg: { height: 38, lineHeight: "38px" },
};

const getStyles = (
  theme: Theme,
  variant: ButtonVariant,
  size: ButtonSize,
  iconOnly: boolean,
  pressed: boolean,
  disabled: boolean,
  hovered: boolean,
): React.CSSProperties => {
  const { height, lineHeight } = sizeStyles[size];

  const base: React.CSSProperties = {
    cursor: disabled ? "not-allowed" : "pointer",
    position: "relative",
    overflow: "hidden",
    padding: iconOnly ? 0 : "0 10px",
    borderRadius: 6,
    display: "inline-flex",
    alignItems: "center",
    justifyContent: "center",
    gap: iconOnly ? 0 : 8,
    fontFamily: "Inter, sans-serif",
    fontSize: 12,
    fontWeight: 500,
    textDecoration: "none",
    border: "1px solid rgba(0, 0, 0, 0.04)",
    boxShadow: pressed
      ? "inset 0 1px 1px rgba(0, 0, 0, 0.12)"
      : "rgba(0, 0, 0, 0.04) 0px 1px 2px",
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

  const variantStyles: Record<ButtonVariant, React.CSSProperties> = {
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
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  function Button(
    {
      variant,
      size,
      children,
      startIcon,
      endIcon,
      iconOnly,
      pressed,
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
    const computedStyles = getStyles(
      theme,
      variant,
      size,
      iconOnly,
      pressed,
      disabled,
      hovered,
    );
    const iconOnlyIcon = startIcon ?? endIcon;

    return (
      <button
        ref={ref}
        disabled={disabled}
        aria-pressed={pressed || undefined}
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
        {iconOnly ? (
          <span
            style={{
              width: 16,
              height: 16,
              display: "flex",
              alignItems: "center",
            }}
          >
            {iconOnlyIcon}
          </span>
        ) : (
          <>
            {startIcon && (
              <span
                style={{
                  width: 16,
                  height: 16,
                  display: "flex",
                  alignItems: "center",
                }}
              >
                {startIcon}
              </span>
            )}
            {children}
            {endIcon && (
              <span
                style={{
                  width: 16,
                  height: 16,
                  display: "flex",
                  alignItems: "center",
                }}
              >
                {endIcon}
              </span>
            )}
          </>
        )}
      </button>
    );
  },
);
Button.displayName = "Button";
