import { styled } from "@mui/material";
import { alpha } from "@mui/material/styles";
import { X } from "lucide-react";
import type React from "react";
import { useEffect, useId } from "react";
import { createPortal } from "react-dom";
import { IconButton } from "../Button/IconButton";

/**
 * This is a reusable Dialog component with optional footer and close button.
 * Width is editable but never less than 280px.
 */

export interface DialogProperties {
  open: boolean;
  onClose: () => void;
  title: React.ReactNode;
  children: React.ReactNode;
  footer?: React.ReactNode;
  showCloseButton?: boolean;
  width?: number | string;
}

const DialogOverlay = styled("div")(({ theme }) => ({
  position: "fixed",
  inset: 0,
  zIndex: theme.zIndex.modal,
  backgroundColor: alpha(theme.palette.common.black, 0.4),
  backdropFilter: "blur(0.5px)",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  padding: theme.spacing(2),
}));

const DialogContainer = styled("div", {
  shouldForwardProp: (prop) => prop !== "dialogWidth",
})<{ dialogWidth: number | string }>(({ theme, dialogWidth }) => ({
  width: dialogWidth,
  minWidth: "min(280px, calc(100vw - 32px))",
  maxWidth: "calc(100vw - 32px)",
  maxHeight: "calc(100vh - 32px)",
  overflow: "hidden",
  backgroundColor: theme.palette.common.white,
  color: theme.palette.common.black,
  borderRadius: 10,
  boxShadow: `0px 30px 70px ${alpha(theme.palette.common.black, 0.24)}`,
  border: `1px solid ${alpha(theme.palette.common.black, 0.12)}`,
  display: "flex",
  flexDirection: "column",
  fontFamily: theme.typography.fontFamily,
}));

const DialogHeader = styled("div")(({ theme }) => ({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  minHeight: 28,
  padding: theme.spacing(1),
  paddingLeft: theme.spacing(2),
  gap: theme.spacing(1),
}));

const DialogTitle = styled("h2")(({ theme }) => ({
  margin: 0,
  fontSize: theme.typography.fontSize * 1.14,
  fontWeight: 700,
  minWidth: 0,
  overflow: "hidden",
  textOverflow: "ellipsis",
  whiteSpace: "nowrap",
  flex: 1,
}));

const DialogContent = styled("div")(({ theme }) => ({
  padding: theme.spacing(2),
  fontSize: theme.typography.fontSize,
  flex: 1,
  minHeight: 0,
  overflow: "auto",
}));

const DialogFooter = styled("div")(({ theme }) => ({
  padding: `${theme.spacing(1)} ${theme.spacing(2)}`,
  display: "flex",
  flexDirection: "column-reverse",
  [theme.breakpoints.up("sm")]: {
    flexDirection: "row",
  },
  gap: theme.spacing(1),
  justifyContent: "flex-end",
}));

export const Dialog = ({
  open,
  onClose,
  title,
  children,
  footer,
  showCloseButton = true,
  width = 300,
}: DialogProperties) => {
  const titleId = useId();

  useEffect(() => {
    if (!open) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        onClose();
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [open, onClose]);

  if (!open) return null;

  return createPortal(
    <DialogOverlay
      role="presentation"
      onMouseDown={(e) => {
        // Close only when clicking the backdrop itself.
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <DialogContainer
        dialogWidth={width}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        onMouseDown={(e) => e.stopPropagation()}
      >
        {/* 1) header */}
        <DialogHeader>
          <DialogTitle id={titleId}>{title}</DialogTitle>

          {showCloseButton && (
            <IconButton
              icon={<X />}
              aria-label="Close dialog"
              onClick={onClose}
            />
          )}
        </DialogHeader>

        {/* 2) content */}
        <DialogContent>{children}</DialogContent>

        {/* 3) footer */}
        {footer != null && <DialogFooter>{footer}</DialogFooter>}
      </DialogContainer>
    </DialogOverlay>,
    document.body,
  );
};

Dialog.displayName = "Dialog";
