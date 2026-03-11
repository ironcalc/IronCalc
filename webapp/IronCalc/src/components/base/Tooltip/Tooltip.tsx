import {
  Tooltip as MuiTooltip,
  type TooltipProps as MuiTooltipProps,
} from "@mui/material";
import { alpha } from "@mui/material/styles";

export interface TooltipProps extends MuiTooltipProps {
  shortcut?: string;
}

const defaultSlotProps: TooltipProps["slotProps"] = {
  popper: {
    modifiers: [{ name: "offset", options: { offset: [0, -2] } }],
  },
  tooltip: {
    sx: (theme) => ({
      fontFamily: "Inter",
      backgroundColor: alpha(theme.palette.grey[900], 0.76),
      color: "common.white",
      "& [data-tooltip-shortcut]": {
        color: "grey.400",
      },
    }),
  },
};

export function Tooltip({
  enterDelay = 700,
  leaveDelay = 0,
  placement = "bottom",
  slotProps,
  title,
  shortcut,
  ...rest
}: TooltipProps) {
  const effectiveTitle =
    shortcut != null && typeof title === "string" ? (
      <>
        {title} <span data-tooltip-shortcut>[{shortcut}]</span>
      </>
    ) : (
      title
    );
  return (
    <MuiTooltip
      enterDelay={enterDelay}
      leaveDelay={leaveDelay}
      placement={placement}
      slotProps={{ ...defaultSlotProps, ...slotProps }}
      title={effectiveTitle}
      {...rest}
    />
  );
}
