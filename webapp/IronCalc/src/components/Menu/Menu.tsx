import {
  cloneElement,
  createContext,
  type ReactElement,
  type ReactNode,
  useEffect,
  useState,
} from "react";
import { createPortal } from "react-dom";

import "./menu.css";
import { useAnchorPosition } from "./useAnchorPosition";
import { useMenuKeyDown } from "./useMenuKeyDown";
import { useMenuPosition } from "./useMenuPosition";

export const MenuContext = createContext<{ close: () => void } | null>(null);

interface MenuTriggerProperties {
  trigger: ReactElement;
  open?: never;
  onClose?: never;
  anchorPosition?: never;
  children: ReactNode;
}

interface MenuControlledProperties {
  trigger?: never;
  open: boolean;
  onClose: () => void;
  anchorPosition: { x: number; y: number };
  children: ReactNode;
}

export type MenuProperties = MenuTriggerProperties | MenuControlledProperties;

export function Menu(props: MenuProperties) {
  const isTriggerMode = props.trigger !== undefined;

  const [uncontrolledOpen, setUncontrolledOpen] = useState(false);
  const open = isTriggerMode ? uncontrolledOpen : props.open;

  const triggerPosition = useMenuPosition(isTriggerMode ? open : false);
  const anchorPosition = useAnchorPosition(
    !isTriggerMode ? open : false,
    !isTriggerMode ? props.anchorPosition : undefined,
  );

  const menuRef = isTriggerMode
    ? triggerPosition.menuRef
    : anchorPosition.menuRef;
  const menuStyle = isTriggerMode
    ? triggerPosition.position
    : anchorPosition.position;

  function close() {
    if (isTriggerMode) {
      setUncontrolledOpen(false);
      triggerPosition.triggerRef.current?.focus();
    } else {
      props.onClose();
    }
  }

  // Close on outside pointer down
  useEffect(() => {
    function handlePointerDown(event: MouseEvent) {
      if (!open) {
        return;
      }
      const target = event.target as Node | null;
      if (!target) {
        return;
      }

      const triggerContains = isTriggerMode
        ? (triggerPosition.triggerRef.current?.contains(target) ?? false)
        : false;

      if (!triggerContains && !(menuRef.current?.contains(target) ?? false)) {
        close();
      }
    }

    document.addEventListener("pointerdown", handlePointerDown, true);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown, true);
    };
  });

  const { handleMenuKeyDown } = useMenuKeyDown(menuRef, close);

  // Focus first item when menu opens so keyboard navigation works immediately.
  useEffect(() => {
    if (!open) return;
    const firstItem = menuRef.current?.querySelector<HTMLButtonElement>(
      ':is([role="menuitem"],[role="menuitemradio"],[role="menuitemcheckbox"]):not([disabled])',
    );
    firstItem?.focus();
  }, [open, menuRef]);

  const menu = open
    ? createPortal(
        <div
          ref={menuRef}
          role="presentation"
          className="ic-menu-wrapper"
          style={menuStyle}
        >
          <div role="menu" className="ic-menu" onKeyDown={handleMenuKeyDown}>
            {props.children}
          </div>
        </div>,
        document.body,
      )
    : null;

  if (!isTriggerMode) {
    return (
      <MenuContext.Provider value={{ close }}>{menu}</MenuContext.Provider>
    );
  }

  const clonedTrigger = cloneElement(
    props.trigger as ReactElement<Record<string, unknown>>,
    {
      ref: triggerPosition.triggerRef,
      onClick: (e: React.MouseEvent) => {
        (
          props.trigger?.props as { onClick?: React.MouseEventHandler }
        ).onClick?.(e);
        setUncontrolledOpen((current) => !current);
      },
      "aria-haspopup": "menu",
      "aria-expanded": open ? "true" : "false",
    },
  );

  return (
    <MenuContext.Provider value={{ close }}>
      {clonedTrigger}
      {menu}
    </MenuContext.Provider>
  );
}

Menu.displayName = "Menu";
