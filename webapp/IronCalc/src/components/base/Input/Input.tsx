import { useTheme } from "@mui/material";
import type { Theme } from "@mui/material/styles";
import { X } from "lucide-react";
import {
  type ChangeEvent,
  type FocusEvent,
  useCallback,
  useId,
  useState,
} from "react";

export type InputSize = "xs" | "sm" | "md" | "lg";

export interface InputProps
  extends Omit<
    React.InputHTMLAttributes<HTMLInputElement | HTMLTextAreaElement>,
    "size"
  > {
  size: InputSize | undefined;
  variant: "outlined" | "filled" | "standard" | "ghost" | undefined;
  label: React.ReactNode | undefined;
  clearable: boolean | undefined;
  startIcon: React.ReactNode | undefined;
  required: boolean | undefined;
  margin: "none" | "dense" | "normal" | undefined;
  multiline: boolean | undefined;
  rows: number | undefined;
  error: boolean | undefined;
  helperText: React.ReactNode | undefined;
  slotProps:
    | {
        input?: {
          startAdornment: React.ReactNode | undefined;
          endAdornment: React.ReactNode | undefined;
        };
      }
    | undefined;
  sx: React.CSSProperties | undefined;
}

const sizeRootStyles: Record<InputSize, { height: number; padding: string }> = {
  xs: { height: 24, padding: "4px 8px" },
  sm: { height: 28, padding: "6px 8px" },
  md: { height: 32, padding: "8px 8px" },
  lg: { height: 38, padding: "11px 8px" },
};

const textareaMinHeights: Record<InputSize, number> = {
  xs: 60,
  sm: 64,
  md: 72,
  lg: 80,
};

const wrapperStyle: React.CSSProperties = {
  display: "flex",
  flexDirection: "column",
  width: "100%",
  gap: 6,
};

function getInputWrapperStyle(
  theme: Theme,
  opts: {
    focused: boolean;
    hovered: boolean;
    error: boolean;
    disabled: boolean;
    ghost: boolean;
    size: InputSize;
  },
): React.CSSProperties {
  const { focused, hovered, error, disabled, ghost, size } = opts;
  const { height, padding } = sizeRootStyles[size];

  const base: React.CSSProperties = {
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
    alignItems: "center",
    height,
    padding,
  };

  if (disabled) {
    return {
      ...base,
      backgroundColor: theme.palette.grey[100],
      color: theme.palette.grey[500],
      cursor: "not-allowed",
      ...(ghost ? {} : { border: `1px solid ${theme.palette.grey[400]}` }),
    };
  }

  if (ghost) {
    return { ...base, border: "none" };
  }

  let borderColor = theme.palette.grey[400];
  if (error) {
    borderColor =
      focused || hovered ? theme.palette.error.main : theme.palette.error.dark;
  } else if (focused) {
    borderColor = theme.palette.primary.main;
  } else if (hovered) {
    borderColor = theme.palette.grey[500];
  }

  return {
    ...base,
    border: `1px solid ${borderColor}`,
    borderWidth: focused ? 1 : 1,
  };
}

function getTextareaStyle(
  theme: Theme,
  opts: { size: InputSize; error: boolean; ghost: boolean; disabled: boolean },
): React.CSSProperties {
  const { size, error, ghost, disabled } = opts;
  const { padding } = sizeRootStyles[size];
  const minHeight = textareaMinHeights[size];

  const base: React.CSSProperties = {
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
  };

  if (disabled) {
    return {
      ...base,
      backgroundColor: theme.palette.grey[100],
      color: theme.palette.grey[500],
      cursor: "not-allowed",
      ...(ghost ? {} : { border: `1px solid ${theme.palette.grey[400]}` }),
    };
  }

  return {
    ...base,
    ...(ghost
      ? { border: "none" }
      : {
          borderWidth: 1,
          borderStyle: "solid",
          borderColor: error
            ? theme.palette.error.main
            : theme.palette.grey[400],
        }),
  };
}

export function Input({
  variant = "outlined",
  size = "md",
  margin = "none",
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
  required = false,
  slotProps = {},
  ...rest
}: InputProps) {
  const theme = useTheme();
  const generatedId = useId();
  const id = idProp ?? generatedId;

  const [focused, setFocused] = useState(false);
  const [hovered, setHovered] = useState(false);

  const hasValue = typeof valueProp === "string" && valueProp.length > 0;
  const showClearButton = clearable && !multiline && !disabled && hasValue;

  const handleClear = useCallback(() => {
    onChange?.({
      target: { value: "" },
    } as ChangeEvent<HTMLInputElement>);
  }, [onChange]);

  const handleFocus = useCallback(
    (e: FocusEvent<HTMLInputElement | HTMLTextAreaElement>) => {
      setFocused(true);
      onFocus?.(e as FocusEvent<HTMLInputElement>);
    },
    [onFocus],
  );

  const handleBlur = useCallback(
    (e: FocusEvent<HTMLInputElement | HTMLTextAreaElement>) => {
      setFocused(false);
      onBlur?.(e as FocusEvent<HTMLInputElement>);
    },
    [onBlur],
  );

  const ghost = variant === "ghost";
  const restSlot = slotProps.input;
  const startAdornment =
    (restSlot as { startAdornment?: React.ReactNode } | undefined)
      ?.startAdornment ?? (startIcon != null ? startIcon : undefined);
  const endAdornmentFromSlot = (
    restSlot as { endAdornment?: React.ReactNode } | undefined
  )?.endAdornment;

  if (multiline) {
    const content = (
      <>
        {label != null && (
          <label
            htmlFor={id}
            style={{
              fontSize: "12px",
              fontFamily: "Inter",
              fontWeight: 500,
              color: theme.palette.text.primary,
              display: "block",
              ...(disabled ? { cursor: "not-allowed" } : {}),
            }}
          >
            {label}
            {required && (
              <span
                aria-hidden
                style={{ marginLeft: 2, color: theme.palette.primary.main }}
              >
                *
              </span>
            )}
          </label>
        )}
        <textarea
          id={id}
          style={getTextareaStyle(theme, {
            size,
            error,
            ghost,
            disabled: !!disabled,
          })}
          disabled={disabled}
          required={required}
          aria-required={required}
          placeholder={placeholder as string | undefined}
          value={valueProp as string | undefined}
          defaultValue={defaultValue as string | undefined}
          onChange={onChange as React.ChangeEventHandler<HTMLTextAreaElement>}
          onBlur={handleBlur as React.FocusEventHandler<HTMLTextAreaElement>}
          onFocus={handleFocus as React.FocusEventHandler<HTMLTextAreaElement>}
          rows={typeof rows === "number" ? rows : 3}
          name={name}
          aria-invalid={error}
          aria-describedby={helperText ? `${id}-helper` : undefined}
        />
        {helperText != null && (
          <div
            id={`${id}-helper`}
            style={{
              fontSize: "12px",
              fontFamily: "Inter",
              marginLeft: 0,
              marginRight: 0,
              color: error ? theme.palette.error.main : theme.palette.grey[500],
            }}
          >
            {helperText}
          </div>
        )}
      </>
    );
    return (
      <div
        style={{
          ...wrapperStyle,
          ...(disabled ? { cursor: "not-allowed" } : {}),
        }}
      >
        {content}
      </div>
    );
  }

  const inputWrapperStyle = getInputWrapperStyle(theme, {
    focused,
    hovered,
    error: !!error,
    disabled: !!disabled,
    ghost,
    size,
  });

  const nativeInputStyle: React.CSSProperties = {
    flex: "1 1 0%",
    minWidth: 0,
    border: "none",
    outline: "none",
    background: "transparent",
    fontFamily: "Inter",
    fontSize: "12px",
    padding: 0,
    color: "inherit",
    boxSizing: "border-box",
  };

  const endAdornment = showClearButton ? (
    <button
      type="button"
      onClick={handleClear}
      onMouseDown={(e) => e.preventDefault()}
      aria-label="Clear"
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: "4px",
        border: "none",
        background: "transparent",
        cursor: "pointer",
        color: "inherit",
      }}
    >
      <X size={14} />
    </button>
  ) : (
    endAdornmentFromSlot
  );

  const textField = (
    <fieldset
      style={{
        ...inputWrapperStyle,
        ...sx,
        margin: 0,
        minWidth: 0,
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {startAdornment != null && (
        <span
          style={{
            display: "flex",
            alignItems: "center",
            flexShrink: 0,
            marginRight: 8,
            maxWidth: 16,
          }}
        >
          {startAdornment}
        </span>
      )}
      <input
        {...rest}
        id={id}
        required={required}
        value={valueProp}
        defaultValue={defaultValue}
        onChange={onChange}
        onBlur={handleBlur}
        onFocus={handleFocus}
        name={name}
        placeholder={placeholder}
        disabled={disabled}
        aria-invalid={error}
        aria-describedby={helperText ? `${id}-helper` : undefined}
        style={{ ...nativeInputStyle, ...rest.style }}
      />
      {endAdornment != null && (
        <span
          style={{
            display: "flex",
            alignItems: "center",
            flexShrink: 0,
            marginLeft: 4,
          }}
        >
          {endAdornment}
        </span>
      )}
    </fieldset>
  );

  if (label != null) {
    return (
      <div
        style={{
          ...wrapperStyle,
          ...(disabled ? { cursor: "not-allowed" } : {}),
        }}
      >
        <label
          htmlFor={id}
          style={{
            fontSize: "12px",
            fontFamily: "Inter",
            fontWeight: 500,
            color: theme.palette.text.primary,
            display: "block",
            ...(disabled ? { cursor: "not-allowed" } : {}),
          }}
        >
          {label}
          {required && (
            <span
              aria-hidden
              style={{ marginLeft: 2, color: theme.palette.primary.main }}
            >
              *
            </span>
          )}
        </label>
        {textField}
        {helperText != null && (
          <div
            id={`${id}-helper`}
            style={{
              fontSize: "12px",
              fontFamily: "Inter",
              marginLeft: 0,
              marginRight: 0,
              color: error ? theme.palette.error.main : theme.palette.grey[500],
            }}
          >
            {helperText}
          </div>
        )}
      </div>
    );
  }

  if (helperText != null) {
    return (
      <div
        style={{
          ...wrapperStyle,
          ...(disabled ? { cursor: "not-allowed" } : {}),
        }}
      >
        {textField}
        <div
          id={`${id}-helper`}
          style={{
            fontSize: "12px",
            fontFamily: "Inter",
            marginLeft: 0,
            marginRight: 0,
            color: error ? theme.palette.error.main : theme.palette.grey[500],
          }}
        >
          {helperText}
        </div>
      </div>
    );
  }

  return textField;
}
