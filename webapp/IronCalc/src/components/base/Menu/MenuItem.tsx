import Popper from "@mui/material/Popper";
import { alpha, styled } from "@mui/material/styles";
import { ChevronRight } from "lucide-react";
import type React from "react";
import { useCallback, useContext, useEffect, useRef, useState } from "react";
import { MENU_PANEL_DATA_ATTR, MenuPanel, SubmenuContext } from "./Menu";

export type MenuItemProps = {
  children: React.ReactNode;
  onClick?: () => void;
  selected?: boolean;
  disabled?: boolean;
  destructive?: boolean;
  startAdornment?: React.ReactNode;
  endAdornment?: React.ReactNode;
  submenu?: React.ReactNode;
  component?: React.ElementType;
};

const SUBMENU_CLOSE_DELAY_MS = 150;

export function MenuItem({
  children,
  onClick,
  selected,
  disabled,
  destructive,
  startAdornment,
  endAdornment,
  submenu,
  component = "button",
}: MenuItemProps) {
  const { openSubmenuAnchor, setOpenSubmenuAnchor } =
    useContext(SubmenuContext);
  const [submenuOpen, setSubmenuOpen] = useState(false);
  const [submenuAnchor, setSubmenuAnchor] = useState<HTMLElement | null>(null);
  const anchorRef = useRef<HTMLDivElement>(null);
  const closeTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearCloseTimeout = useCallback(() => {
    if (closeTimeoutRef.current) {
      clearTimeout(closeTimeoutRef.current);
      closeTimeoutRef.current = null;
    }
  }, []);

  const handleSubmenuOpen = () => {
    if (disabled || !submenu) return;
    clearCloseTimeout();
    const el = anchorRef.current;
    setSubmenuAnchor(el);
    setSubmenuOpen(true);
    setOpenSubmenuAnchor(el);
  };

  const handleSubmenuClose = () => {
    clearCloseTimeout();
    closeTimeoutRef.current = setTimeout(() => {
      setSubmenuOpen(false);
      setSubmenuAnchor(null);
      setOpenSubmenuAnchor(null);
      closeTimeoutRef.current = null;
    }, SUBMENU_CLOSE_DELAY_MS);
  };

  useEffect(() => {
    if (
      submenuOpen &&
      openSubmenuAnchor != null &&
      openSubmenuAnchor !== anchorRef.current
    ) {
      clearCloseTimeout();
      setSubmenuOpen(false);
      setSubmenuAnchor(null);
    }
  }, [submenuOpen, openSubmenuAnchor, clearCloseTimeout]);

  useEffect(
    () => () => {
      clearCloseTimeout();
    },
    [clearCloseTimeout],
  );

  const effectiveEndAdornment =
    submenu != null ? (
      <>
        {endAdornment}
        <ChevronRight style={{ width: 16, height: 16, marginLeft: 4 }} />
      </>
    ) : (
      endAdornment
    );

  const item = (
    <MenuItemWrapper
      as={component}
      type={component === "button" ? "button" : undefined}
      onClick={disabled ? undefined : onClick}
      data-selected={selected ? "" : undefined}
      disabled={disabled}
      $destructive={destructive}
    >
      {startAdornment != null && (
        <MenuItemStartAdornment>{startAdornment}</MenuItemStartAdornment>
      )}
      <MenuItemText>{children}</MenuItemText>
      {effectiveEndAdornment != null && (
        <MenuItemEndAdornment>{effectiveEndAdornment}</MenuItemEndAdornment>
      )}
    </MenuItemWrapper>
  );

  if (submenu != null) {
    return (
      <>
        <SubMenuAnchor
          ref={anchorRef}
          onMouseEnter={handleSubmenuOpen}
          onMouseLeave={handleSubmenuClose}
        >
          {item}
        </SubMenuAnchor>
        {submenuAnchor && (
          <SubmenuPopper
            open={submenuOpen}
            anchorEl={submenuAnchor}
            placement="right-start"
            keepMounted={false}
            modifiers={[{ name: "offset", options: { offset: [-4, 0] } }]}
          >
            <MenuPanel
              {...{ [MENU_PANEL_DATA_ATTR]: "" }}
              onMouseEnter={handleSubmenuOpen}
              onMouseLeave={handleSubmenuClose}
            >
              {submenu}
            </MenuPanel>
          </SubmenuPopper>
        )}
      </>
    );
  }

  return item;
}

const MenuItemWrapper = styled("button", {
  shouldForwardProp: (prop) => prop !== "$destructive",
})<{ $destructive?: boolean }>`
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  width: calc(100% - 8px);
  min-width: 172px;
  margin: 0 4px;
  border-radius: 4px;
  padding: 8px;
  height: 32px;
  border: none;
  cursor: pointer;
  background: transparent;
  color: ${({ theme, $destructive }) =>
    $destructive ? theme.palette.error.main : theme.palette.common.black};
  font-family: inherit;
  text-align: left;

  svg {
    color: ${({ theme, $destructive }) =>
      $destructive ? theme.palette.error.main : "inherit"};
  }

  &:hover:not(:disabled) {
    background-color: ${({ theme, $destructive }) =>
      $destructive
        ? alpha(theme.palette.error.main, 0.1)
        : theme.palette.action.hover};
  }
  &:disabled {
    cursor: default;
    opacity: 0.6;
  }
  &[data-selected] {
    background-color: ${({ theme, $destructive }) =>
      $destructive
        ? alpha(theme.palette.error.main, 0.1)
        : theme.palette.action.selected};
  }
  &[data-selected] > span:first-of-type {
    color: inherit;
  }
  ${({ $destructive }) =>
    $destructive &&
    `
    & span, & > * {
      color: inherit;
    }
  `}
`;

const MenuItemStartAdornment = styled("span")`
  margin-right: 8px;
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  color: ${({ theme }) => theme.palette.grey[700]};
  svg {
    width: 16px;
    height: 16px;
  }
`;

const MenuItemText = styled("span")`
  flex: 1;
  color: ${({ theme }) => theme.palette.text.primary};
`;

const MenuItemEndAdornment = styled("span")`
  margin-left: 20px;
  color: ${({ theme }) => theme.palette.grey[500]};
  display: inline-flex;
  align-items: center;
`;

const SubMenuAnchor = styled("div")``;

const SubmenuPopper = styled(Popper)`
  z-index: 1300;
  &[data-popper-placement] {
    pointer-events: auto;
  }
`;
