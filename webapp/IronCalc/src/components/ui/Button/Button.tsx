import {
  Button as MuiButton,
  type ButtonProps as MuiButtonProps,
  useTheme,
} from "@mui/material";
import type { Theme } from "@mui/material/styles";

export type ButtonVariant =
  | "primary"
  | "secondary"
  | "outline"
  | "ghost"
  | "destructive";
export type ButtonSize = "xs" | "sm" | "md" | "lg";

export interface ButtonProps extends Omit<MuiButtonProps, "variant" | "size"> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  iconOnly?: boolean;
}

const variantToMui: Record<
  ButtonVariant,
  {
    variant: "contained" | "outlined" | "text";
    color: "primary" | "secondary" | "error" | "inherit";
  }
> = {
  primary: { variant: "contained", color: "primary" },
  secondary: { variant: "contained", color: "secondary" },
  outline: { variant: "outlined", color: "primary" },
  ghost: { variant: "text", color: "inherit" },
  destructive: { variant: "contained", color: "error" },
};

const sizeToMui: Record<ButtonSize, MuiButtonProps["size"]> = {
  xs: "small",
  sm: "small",
  md: "medium",
  lg: "large",
};

const sizeSx: Record<ButtonSize, { height: number; lineHeight: string }> = {
  xs: { height: 24, lineHeight: "24px" },
  sm: { height: 28, lineHeight: "28px" },
  md: { height: 32, lineHeight: "32px" },
  lg: { height: 38, lineHeight: "38px" },
};

const baseSx = {
  cursor: "pointer",
  position: "relative",
  overflow: "hidden",
  padding: "0 10px",
  borderRadius: "6px",
  display: "flex",
  alignItems: "center",
  gap: "8px",
  fontFamily: "Inter",
  fontSize: "12px",
  textTransform: "none",
  border: "1px solid rgba(0, 0, 0, 0.04)",
  boxShadow: "rgba(0, 0, 0, 0.04) 0px 1px 2px",
  "&:hover": {
    boxShadow: "rgba(0, 0, 0, 0.04) 0px 1px 2px",
  },
  "&:active": {
    boxShadow: "inset 0 1px 2px rgba(0, 0, 0, 0.08)",
    "&::after": {
      content: '""',
      position: "absolute",
      inset: 0,
      borderRadius: "5px",
      backgroundColor: "rgba(0, 0, 0, 0.04)",
      pointerEvents: "none",
    },
  },
  "& .MuiButton-startIcon": {
    margin: 0,
    "& svg": { width: 16, height: 16 },
  },
  "& .MuiButton-endIcon": {
    margin: 0,
    "& svg": { width: 16, height: 16 },
  },
} as const;

const getBaseSx = (size: ButtonSize) => ({ ...baseSx, ...sizeSx[size] });

const getIconOnlySx = (size: ButtonSize): Record<string, unknown> => {
  const height = sizeSx[size].height;
  return {
    minWidth: height,
    width: height,
    padding: 0,
    gap: 0,
  };
};

const getVariantSx = (
  theme: Theme,
  variant: ButtonVariant,
): Record<string, unknown> => {
  switch (variant) {
    case "secondary":
      return {
        backgroundColor: theme.palette.grey[200],
        color: theme.palette.common.black,
        "&:hover": {
          borderColor: theme.palette.grey[400],
          backgroundColor: theme.palette.grey[300],
          boxShadow: "none",
        },
      };
    case "outline":
      return {
        borderColor: theme.palette.grey[300],
        color: theme.palette.common.black,
        "&:hover": {
          borderColor: theme.palette.grey[400],
          backgroundColor: theme.palette.grey[50],
        },
      };
    case "ghost":
      return {
        border: "none",
        boxShadow: "none",
        color: theme.palette.common.black,
        "&:hover": {
          backgroundColor: theme.palette.grey[100],
        },
      };
    case "destructive":
      return {
        backgroundColor: theme.palette.error.main,
        color: theme.palette.error.contrastText,
        borderColor: theme.palette.error.main,
        "&:hover": {
          backgroundColor: theme.palette.error.dark,
          borderColor: theme.palette.error.dark,
          boxShadow: "none",
        },
      };
    default:
      return {};
  }
};

const getDisabledSx = (theme: Theme): Record<string, unknown> => ({
  "&.Mui-disabled": {
    backgroundColor: theme.palette.grey[200],
    color: theme.palette.action.disabled,
    borderColor: theme.palette.grey[300],
    boxShadow: "none",
  },
});

export function Button({
  variant = "primary",
  size = "md",
  children,
  startIcon,
  endIcon,
  iconOnly = false,
  sx,
  ...rest
}: ButtonProps) {
  const theme = useTheme();
  const { variant: muiVariant, color } = variantToMui[variant];
  const resolvedSx = typeof sx === "function" ? sx(theme) : (sx ?? {});
  const iconOnlyIcon = startIcon ?? endIcon;
  const combinedSx = {
    ...getBaseSx(size),
    ...getVariantSx(theme, variant),
    ...getDisabledSx(theme),
    ...(iconOnly ? getIconOnlySx(size) : {}),
    ...(typeof resolvedSx === "object" &&
    resolvedSx !== null &&
    !Array.isArray(resolvedSx)
      ? resolvedSx
      : {}),
  };
  return (
    <MuiButton
      variant={muiVariant}
      color={color}
      size={sizeToMui[size]}
      disableRipple
      startIcon={iconOnly ? iconOnlyIcon : startIcon}
      endIcon={iconOnly ? undefined : endIcon}
      sx={combinedSx}
      {...rest}
    >
      {iconOnly ? undefined : children}
    </MuiButton>
  );
}
