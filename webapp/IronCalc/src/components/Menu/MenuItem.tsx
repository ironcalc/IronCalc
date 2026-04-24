import { Check, ChevronRight } from "lucide-react";
import { type ReactNode, useContext, useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { MenuContext } from "./Menu";
import { useAnchorPosition } from "./useAnchorPosition";
import { useMenuKeyDown } from "./useMenuKeyDown";

// Tracks the setter of whichever submenu is currently open so a new one can
// close it immediately without waiting for the hide timer.
let activeSetOpen: ((open: boolean) => void) | null = null;

export interface MenuItemProperties {
  onClick?: () => void;
  disabled?: boolean;
  destructive?: boolean;
  icon?: ReactNode;
  secondaryText?: ReactNode;
  checked?: boolean;
  children: ReactNode;
}

export function MenuItem({
  onClick,
  disabled = false,
  destructive = false,
  icon,
  secondaryText,
  checked,
  children,
}: MenuItemProperties) {
  const menu = useContext(MenuContext);
  const isRadio = checked !== undefined;

  function handleClick() {
    onClick?.();
    menu?.close();
  }

  return (
    <button
      type="button"
      role={isRadio ? "menuitemradio" : "menuitem"}
      {...(isRadio && { "aria-checked": checked })}
      className={[
        "ic-menu-item",
        destructive && "destructive",
        disabled && "disabled",
      ]
        .filter(Boolean)
        .join(" ")}
      disabled={disabled}
      onClick={handleClick}
    >
      {isRadio && (
        <span
          className="ic-menu-item-icon"
          aria-hidden="true"
          style={{ visibility: checked ? "visible" : "hidden" }}
        >
          <Check />
        </span>
      )}
      {icon && (
        <span className="ic-menu-item-icon" aria-hidden="true">
          {icon}
        </span>
      )}
      <span className="ic-menu-item-label">{children}</span>
      {secondaryText ? (
        <span className="ic-menu-item-secondary">{secondaryText}</span>
      ) : null}
    </button>
  );
}

MenuItem.displayName = "MenuItem";

export interface MenuItemWithSubmenuProps {
  icon?: ReactNode;
  children: ReactNode;
  submenu: ReactNode;
}

export function MenuItemWithSubmenu({
  icon,
  children,
  submenu,
}: MenuItemWithSubmenuProps) {
  const parentMenu = useContext(MenuContext);
  const [open, setOpen] = useState(false);
  const [anchor, setAnchor] = useState<
    { x: number; y: number; flipX?: number } | undefined
  >();
  const itemRef = useRef<HTMLButtonElement>(null);
  const closeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const focusOnOpenRef = useRef(false);

  const { menuRef, position } = useAnchorPosition(open, anchor);
  const { handleMenuKeyDown } = useMenuKeyDown(
    menuRef,
    () => {
      setOpen(false);
      itemRef.current?.focus();
    },
    true,
  );

  useEffect(() => {
    if (!open || !focusOnOpenRef.current) return;
    focusOnOpenRef.current = false;
    const firstItem = menuRef.current?.querySelector<HTMLButtonElement>(
      ':is([role="menuitem"],[role="menuitemradio"],[role="menuitemcheckbox"]):not([disabled])',
    );
    firstItem?.focus();
  }, [open, menuRef]);

  function show() {
    if (closeTimerRef.current) clearTimeout(closeTimerRef.current);
    if (activeSetOpen && activeSetOpen !== setOpen) activeSetOpen(false);
    activeSetOpen = setOpen;
    const rect = itemRef.current?.getBoundingClientRect();
    if (rect)
      setAnchor({ x: rect.right + 4, y: rect.top - 4, flipX: rect.left - 4 });
    setOpen(true);
  }

  function scheduleHide() {
    closeTimerRef.current = setTimeout(() => {
      setOpen(false);
      if (activeSetOpen === setOpen) activeSetOpen = null;
    }, 150);
  }

  function cancelHide() {
    if (closeTimerRef.current) clearTimeout(closeTimerRef.current);
  }

  function closeAll() {
    setOpen(false);
    parentMenu?.close();
  }

  return (
    <>
      <button
        ref={itemRef}
        type="button"
        role="menuitem"
        className="ic-menu-item"
        onMouseEnter={show}
        onMouseLeave={scheduleHide}
        onKeyDown={(e) => {
          if (e.key === "ArrowRight" || e.key === "Enter") {
            e.preventDefault();
            focusOnOpenRef.current = true;
            show();
          }
        }}
        aria-haspopup="menu"
        aria-expanded={open}
      >
        {icon ? (
          <span className="ic-menu-item-icon" aria-hidden="true">
            {icon}
          </span>
        ) : null}
        <span className="ic-menu-item-label">{children}</span>
        <span className="ic-menu-item-icon" aria-hidden="true">
          <ChevronRight />
        </span>
      </button>

      {open
        ? createPortal(
            <MenuContext.Provider value={{ close: closeAll }}>
              <div
                ref={menuRef}
                role="presentation"
                className="ic-menu-wrapper"
                style={position}
              >
                <div
                  role="menu"
                  className="ic-menu"
                  onMouseEnter={cancelHide}
                  onMouseLeave={scheduleHide}
                  onKeyDown={(e) => {
                    e.stopPropagation();
                    handleMenuKeyDown(e);
                  }}
                >
                  {submenu}
                </div>
              </div>
            </MenuContext.Provider>,
            document.body,
          )
        : null}
    </>
  );
}

MenuItemWithSubmenu.displayName = "MenuItemWithSubmenu";
