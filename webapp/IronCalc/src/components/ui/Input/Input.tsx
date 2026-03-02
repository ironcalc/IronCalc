import {
  IconButton,
  InputAdornment,
  type InputBaseProps,
  type StandardTextFieldProps,
  TextField,
} from "@mui/material";
import type { SxProps, Theme } from "@mui/material/styles";
import { styled } from "@mui/material/styles";
import { X } from "lucide-react";
import { useId } from "react";

export type InputSize = "xs" | "sm" | "md" | "lg";

export interface InputProps
  extends Omit<StandardTextFieldProps, "size" | "variant"> {
  size?: InputSize;
  variant?: "outlined" | "filled" | "standard" | "ghost";
  label?: React.ReactNode;
  clearable?: boolean;
  startIcon?: React.ReactNode;
  onSubmit?: () => void;
  onCancel?: () => void;
  required?: boolean;
}

const sizeToMui: Record<InputSize, InputBaseProps["size"]> = {
  xs: "small",
  sm: "small",
  md: "medium",
  lg: "medium",
};

/** Fixed heights + padding for single-line (aligned with Button sizeSx) */
const sizeRootStyles: Record<InputSize, { height: number; padding: string }> = {
  xs: { height: 24, padding: "4px 8px" },
  sm: { height: 28, padding: "6px 8px" },
  md: { height: 32, padding: "8px 8px" },
  lg: { height: 38, padding: "11px 8px" },
};

/** Min heights for textarea (no fixed height so it can grow) */
const textareaMinHeights: Record<InputSize, number> = {
  xs: 60,
  sm: 64,
  md: 72,
  lg: 80,
};

const StyledTextField = styled(TextField, {
  shouldForwardProp: (prop) => prop !== "$ghost",
})<{ $ghost?: boolean }>(({ theme, $ghost }) => ({
  minWidth: 0,
  maxWidth: "100%",
  width: "100%",
  "& .MuiInputBase-root": {
    width: "100%",
    minWidth: 0,
    maxWidth: "100%",
    margin: 0,
    fontFamily: "Inter",
    fontSize: "12px",
    borderRadius: 6,
    boxSizing: "border-box",
    overflow: "hidden",
    outline: "none",
    display: "flex",
  },
  "& .MuiInputBase-root > *": {
    minWidth: 0,
  },
  "& .MuiInputBase-root:focus-within": {
    outline: "none",
  },
  "& .MuiInputBase-input": {
    padding: "0px",
    minWidth: 0,
    width: "100%",
    flex: "1 1 0%",
    boxSizing: "border-box",
  },
  "& .MuiOutlinedInput-notchedOutline": {
    borderRadius: 6,
    ...($ghost ? { border: "none" } : { borderColor: theme.palette.grey[400] }),
  },
  ...(!$ghost && {
    "& .MuiInputBase-root:hover .MuiOutlinedInput-notchedOutline": {
      borderColor: theme.palette.grey[500],
    },
    "& .MuiInputBase-root.Mui-focused .MuiOutlinedInput-notchedOutline": {
      borderColor: theme.palette.primary.main,
      borderWidth: "1px",
    },
    "& .MuiInputBase-root.Mui-focused:hover .MuiOutlinedInput-notchedOutline": {
      borderColor: theme.palette.primary.main,
      borderWidth: "1px",
    },
    "& .MuiInputBase-root.Mui-error:hover .MuiOutlinedInput-notchedOutline": {
      borderColor: theme.palette.error.dark,
    },
    "& .MuiInputBase-root.Mui-error.Mui-focused .MuiOutlinedInput-notchedOutline":
      {
        borderColor: theme.palette.error.main,
        borderWidth: "1px",
      },
    "& .MuiInputBase-root.Mui-error.Mui-focused:hover .MuiOutlinedInput-notchedOutline":
      {
        borderColor: theme.palette.error.main,
        borderWidth: "1px",
      },
    "& .MuiInputBase-root.Mui-disabled .MuiOutlinedInput-notchedOutline": {
      borderColor: theme.palette.grey[400],
    },
  }),
  "& .MuiInputBase-root.Mui-disabled": {
    backgroundColor: theme.palette.grey[100],
    color: theme.palette.grey[500],
    cursor: "not-allowed",
  },
  "& .MuiInputBase-root.Mui-disabled .MuiInputBase-input": {
    color: theme.palette.grey[500],
    WebkitTextFillColor: theme.palette.grey[500],
    cursor: "not-allowed",
  },
  "& .MuiFormHelperText-root": {
    marginLeft: 0,
    marginRight: 0,
    color: theme.palette.grey[500],
  },
  "& .MuiFormHelperText-root.Mui-error": {
    color: theme.palette.error.main,
  },
}));

const Label = styled("label")(({ theme }) => ({
  fontSize: "12px",
  fontFamily: "Inter",
  fontWeight: 500,
  color: theme.palette.text.primary,
  display: "block",
}));

const RequiredAsterisk = styled("span")(({ theme }) => ({
  marginLeft: 2,
  color: theme.palette.primary.main,
}));

const Wrapper = styled("div")({
  display: "flex",
  flexDirection: "column",
  width: "100%",
  gap: 6,
});

/** Border colors aligned with StyledTextField (MuiOutlinedInput-notchedOutline) */
const StyledTextarea = styled("textarea")<{
  $size: InputSize;
  $error?: boolean;
  $ghost?: boolean;
}>(({ theme, $size, $error, $ghost }) => {
  const padding = sizeRootStyles[$size].padding;
  const minHeight = textareaMinHeights[$size];
  return {
    width: "100%",
    minHeight,
    padding,
    fontFamily: "Inter",
    fontSize: "12px",
    lineHeight: 1.5,
    borderRadius: 6,
    boxSizing: "border-box",
    resize: "vertical",
    outline: "none",
    ...($ghost
      ? { border: "none" }
      : {
          borderWidth: 1,
          borderStyle: "solid",
          borderColor: $error
            ? theme.palette.error.main
            : theme.palette.grey[400],
        }),
    "&::placeholder": {
      color: theme.palette.text.disabled,
    },
    ...(!$ghost && {
      "&:hover": {
        borderColor: $error
          ? theme.palette.error.dark
          : theme.palette.grey[500],
      },
      "&:focus": {
        borderColor: $error
          ? theme.palette.error.main
          : theme.palette.primary.main,
        borderWidth: 1,
      },
      "&:focus:hover": {
        borderColor: $error
          ? theme.palette.error.main
          : theme.palette.primary.main,
        borderWidth: 1,
      },
    }),
    "&:disabled": {
      backgroundColor: theme.palette.grey[100],
      color: theme.palette.grey[500],
      WebkitTextFillColor: theme.palette.grey[500],
      cursor: "not-allowed",
      ...(!$ghost && { borderColor: theme.palette.grey[400] }),
    },
  };
});

const HelperText = styled("div")<{ $error?: boolean }>(({ theme, $error }) => ({
  fontSize: "12px",
  fontFamily: "Inter",
  marginLeft: 0,
  marginRight: 0,
  color: $error ? theme.palette.error.main : theme.palette.grey[500],
}));

export function Input({
  variant = "outlined",
  size = "md",
  margin = "none",
  fullWidth = true,
  label,
  id: idProp,
  multiline = false,
  sx,
  error = false,
  helperText,
  disabled,
  placeholder,
  value: valueProp,
  defaultValue,
  onChange,
  onBlur,
  onFocus,
  rows,
  name,
  clearable = false,
  startIcon,
  onSubmit,
  onCancel,
  onKeyDown: onKeyDownProp,
  required = false,
  ...rest
}: InputProps) {
  const generatedId = useId();
  const id = idProp ?? generatedId;

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !multiline) {
      onSubmit?.();
      e.preventDefault();
    } else if (e.key === "Escape") {
      onCancel?.();
      (document.activeElement as HTMLElement)?.blur();
    }
    onKeyDownProp?.(e as React.KeyboardEvent<HTMLInputElement>);
  };

  const hasValue = typeof valueProp === "string" && valueProp.length > 0;
  const showClearButton = clearable && !multiline && !disabled && hasValue;

  const handleClear = () => {
    onChange?.({
      target: { value: "" },
    } as React.ChangeEvent<HTMLInputElement>);
  };

  if (multiline) {
    const content = (
      <>
        {label != null && (
          <Label htmlFor={id}>
            {label}
            {required && <RequiredAsterisk aria-hidden>*</RequiredAsterisk>}
          </Label>
        )}
        <StyledTextarea
          id={id}
          $size={size}
          $error={error}
          $ghost={variant === "ghost"}
          disabled={disabled}
          required={required}
          aria-required={required}
          placeholder={placeholder as string | undefined}
          value={valueProp as string | undefined}
          defaultValue={defaultValue as string | undefined}
          onChange={onChange as React.ChangeEventHandler<HTMLTextAreaElement>}
          onBlur={onBlur as React.FocusEventHandler<HTMLTextAreaElement>}
          onFocus={onFocus as React.FocusEventHandler<HTMLTextAreaElement>}
          onKeyDown={handleKeyDown}
          rows={typeof rows === "number" ? rows : 3}
          name={name}
          aria-invalid={error}
          aria-describedby={helperText ? `${id}-helper` : undefined}
        />
        {helperText != null && (
          <HelperText id={`${id}-helper`} $error={error}>
            {helperText}
          </HelperText>
        )}
      </>
    );
    return (
      <Wrapper style={fullWidth ? undefined : { width: "auto" }}>
        {content}
      </Wrapper>
    );
  }

  const rootSizeStyles = {
    "& .MuiInputBase-root": sizeRootStyles[size],
  };
  const resolvedSx: SxProps<Theme> = sx
    ? ([rootSizeStyles, sx] as SxProps<Theme>)
    : rootSizeStyles;

  const { slotProps: restSlotProps, ...restWithoutSlotProps } = rest;
  const restInputSlot =
    restSlotProps?.input && typeof restSlotProps.input === "object"
      ? restSlotProps.input
      : {};
  const inputSlotProps = {
    ...restInputSlot,
    startAdornment:
      (restInputSlot as { startAdornment?: React.ReactNode }).startAdornment ??
      (startIcon != null ? (
        <InputAdornment
          position="start"
          sx={{ "& svg": { width: 16, height: 16 } }}
        >
          {startIcon}
        </InputAdornment>
      ) : undefined),
    endAdornment: showClearButton ? (
      <InputAdornment position="end" disablePointerEvents={false}>
        <IconButton
          size="small"
          onClick={handleClear}
          onMouseDown={(e) => e.preventDefault()}
          aria-label="Clear"
          sx={{ padding: "4px" }}
        >
          <X size={14} />
        </IconButton>
      </InputAdornment>
    ) : (
      (("endAdornment" in restInputSlot
        ? restInputSlot.endAdornment
        : undefined) as React.ReactNode)
    ),
  };
  const textField = (
    <StyledTextField
      variant={variant === "ghost" ? "outlined" : variant}
      $ghost={variant === "ghost"}
      size={sizeToMui[size]}
      required={required}
      margin={margin}
      fullWidth={fullWidth}
      id={id}
      error={error}
      helperText={helperText}
      disabled={disabled}
      placeholder={placeholder}
      value={valueProp}
      defaultValue={defaultValue}
      onChange={onChange}
      onBlur={onBlur}
      onFocus={onFocus}
      onKeyDown={handleKeyDown}
      name={name}
      slotProps={{ input: inputSlotProps }}
      sx={resolvedSx}
      {...restWithoutSlotProps}
    />
  );

  if (label != null) {
    return (
      <Wrapper style={fullWidth ? undefined : { width: "auto" }}>
        <Label htmlFor={id}>
          {label}
          {required && <RequiredAsterisk aria-hidden>*</RequiredAsterisk>}
        </Label>
        {textField}
      </Wrapper>
    );
  }

  return textField;
}
