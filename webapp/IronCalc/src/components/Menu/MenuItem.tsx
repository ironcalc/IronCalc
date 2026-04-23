import { type ReactNode, useContext } from "react";

import { MenuContext } from "./Menu";

export interface MenuItemProperties {
  onClick?: () => void;
  disabled?: boolean;
  destructive?: boolean;
  icon?: ReactNode;
  secondaryText?: ReactNode;
  children: ReactNode;
}

export function MenuItem({
  onClick,
  disabled = false,
  destructive = false,
  icon,
  secondaryText,
  children,
}: MenuItemProperties) {
  const menu = useContext(MenuContext);

  function handleClick() {
    onClick?.();
    menu?.close();
  }

  return (
    <button
      type="button"
      role="menuitem"
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
      {icon ? (
        <span className="ic-menu-item-icon" aria-hidden="true">
          {icon}
        </span>
      ) : null}
      <span className="ic-menu-item-label">{children}</span>
      {secondaryText ? (
        <span className="ic-menu-item-secondary">{secondaryText}</span>
      ) : null}
    </button>
  );
}

MenuItem.displayName = "MenuItem";
