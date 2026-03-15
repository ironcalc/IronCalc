import { alpha, styled } from "@mui/material/styles";
import { ChevronRight } from "lucide-react";
import type React from "react";
import {
  useCallback,
  useContext,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { createPortal } from "react-dom";
import { MENU_PANEL_DATA_ATTR, MenuPanel, SubmenuContext } from "./Menu";

export type MenuItemProps = {
  children: React.ReactNode;
  component?: React.ElementType;
  onClick: () => void;
  selected: boolean;
  disabled: boolean;
  destructive: boolean;
  startAdornment: React.ReactNode | null;
  endAdornment: React.ReactNode | null;
  submenu: React.ReactNode | null;
};

const SUBMENU_CLOSE_DELAY_MS = 150;
const SUBMENU_OFFSET: [number, number] = [-4, 0];

export function MenuItem(props: MenuItemProps) {
  const {
    children,
    component: componentProp = "button",
    onClick,
    selected,
    disabled,
    destructive,
    startAdornment,
    endAdornment,
    submenu,
  } = props;

  const isButton = componentProp === "button";

  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      if (disabled) {
        e.preventDefault();
        e.stopPropagation();
        return;
      }
      onClick();
    },
    [disabled, onClick],
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (disabled && (e.key === "Enter" || e.key === " ")) {
        e.preventDefault();
        e.stopPropagation();
      }
    },
    [disabled],
  );

  const { openSubmenuAnchor, setOpenSubmenuAnchor } =
    useContext(SubmenuContext);
  // ✂️ Merged submenuOpen + submenuAnchor into one nullable state
  const [submenuAnchor, setSubmenuAnchor] = useState<HTMLElement | null>(null);
  const [submenuPosition, setSubmenuPosition] = useState({ top: 0, left: 0 });
  const anchorRef = useRef<HTMLDivElement>(null);
  const closeTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearCloseTimeout = useCallback(() => {
    if (closeTimeoutRef.current) {
      clearTimeout(closeTimeoutRef.current);
      closeTimeoutRef.current = null;
    }
  }, []);

  const openSubmenu = () => {
    if (disabled || !submenu) return;
    clearCloseTimeout();
    const el = anchorRef.current;
    setSubmenuAnchor(el);
    setOpenSubmenuAnchor(el);
  };

  const closeSubmenu = () => {
    clearCloseTimeout();
    closeTimeoutRef.current = setTimeout(() => {
      setSubmenuAnchor(null);
      setOpenSubmenuAnchor(null);
      closeTimeoutRef.current = null;
    }, SUBMENU_CLOSE_DELAY_MS);
  };

  const updateSubmenuPosition = useCallback(() => {
    const anchor = anchorRef.current;
    if (!anchor) return;
    const rect = anchor.getBoundingClientRect();
    const [skidding, distance] = SUBMENU_OFFSET;
    setSubmenuPosition({
      left: rect.right + distance,
      top: rect.top + skidding,
    });
  }, []);

  // Close this submenu if another item opened its own
  useEffect(() => {
    if (submenuAnchor && openSubmenuAnchor !== anchorRef.current) {
      clearCloseTimeout();
      setSubmenuAnchor(null);
    }
  }, [submenuAnchor, openSubmenuAnchor, clearCloseTimeout]);

  useLayoutEffect(() => {
    if (!submenuAnchor) return;
    updateSubmenuPosition();
    const anchor = anchorRef.current;
    if (!anchor) return;
    const resizeObserver = new ResizeObserver(updateSubmenuPosition);
    resizeObserver.observe(anchor);
    window.addEventListener("scroll", updateSubmenuPosition, true);
    window.addEventListener("resize", updateSubmenuPosition);
    return () => {
      resizeObserver.disconnect();
      window.removeEventListener("scroll", updateSubmenuPosition, true);
      window.removeEventListener("resize", updateSubmenuPosition);
    };
  }, [submenuAnchor, updateSubmenuPosition]);

  useEffect(() => () => clearCloseTimeout(), [clearCloseTimeout]);

  const item = (
    <MenuItemWrapper
      component={componentProp}
      type={isButton ? "button" : undefined}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      data-selected={selected ? "" : undefined}
      aria-disabled={disabled ? true : undefined}
      tabIndex={!isButton && disabled ? -1 : undefined}
      {...(isButton ? { disabled } : {})}
      $destructive={destructive}
      onMouseEnter={submenu ? openSubmenu : undefined}
      onMouseLeave={submenu ? closeSubmenu : undefined}
    >
      {startAdornment && (
        <MenuItemStartAdornment>{startAdornment}</MenuItemStartAdornment>
      )}
      <MenuItemText>{children}</MenuItemText>
      {(endAdornment || submenu) && (
        <MenuItemEndAdornment>
          {endAdornment}
          {submenu && (
            <ChevronRight style={{ width: 16, height: 16, marginLeft: 4 }} />
          )}
        </MenuItemEndAdornment>
      )}
    </MenuItemWrapper>
  );

  if (!submenu) return item;

  return (
    <div ref={anchorRef}>
      {item}
      {submenuAnchor &&
        createPortal(
          <SubmenuPositionedWrapper style={submenuPosition}>
            <MenuPanel
              {...{ [MENU_PANEL_DATA_ATTR]: "" }}
              onMouseEnter={openSubmenu}
              onMouseLeave={closeSubmenu}
            >
              {submenu}
            </MenuPanel>
          </SubmenuPositionedWrapper>,
          document.body,
        )}
    </div>
  );
}

const MenuItemWrapper = styled("button", {
  shouldForwardProp: (prop) => prop !== "$destructive",
})<{ $destructive?: boolean; component?: React.ElementType }>`
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

  &:hover:not(:disabled):not([aria-disabled="true"]) {
    background-color: ${({ theme, $destructive }) =>
      $destructive
        ? alpha(theme.palette.error.main, 0.1)
        : theme.palette.action.hover};
  }
  &:disabled,
  &[aria-disabled="true"] {
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

const SubmenuPositionedWrapper = styled("div")`
  position: fixed;
  z-index: 1300;
  pointer-events: auto;
`;
