import {
  type ButtonHTMLAttributes,
  type ReactNode,
  useEffect,
  useRef,
} from "react";
import "./common.css";

interface StyledMenuProps {
  open: boolean;
  onClose: () => void;
  children: ReactNode;
  anchorPosition?: { top: number; left: number };
}

export function StyledMenu({
  open,
  onClose,
  children,
  anchorPosition,
}: StyledMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) {
      return;
    }

    function handleKeyDown(event: KeyboardEvent) {
      event.stopPropagation();
      if (event.key === "Escape") {
        onClose();
      }
    }

    document.addEventListener("keydown", handleKeyDown, true);
    return () => {
      document.removeEventListener("keydown", handleKeyDown, true);
    };
  }, [open, onClose]);

  useEffect(() => {
    if (!open) {
      return;
    }

    menuRef.current?.focus();
  }, [open]);

  if (!open || !anchorPosition) {
    return null;
  }

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: FIXME
    <div className="ic-context-menu-backdrop" onMouseDown={onClose}>
      <div
        ref={menuRef}
        className="ic-context-menu"
        role="menu"
        tabIndex={-1}
        style={{
          top: anchorPosition.top,
          left: anchorPosition.left,
        }}
        onMouseDown={(event) => {
          event.stopPropagation();
        }}
      >
        {children}
      </div>
    </div>
  );
}

type StyledMenuItemProps = ButtonHTMLAttributes<HTMLButtonElement>;

export function StyledMenuItem({
  className = "",
  children,
  onClick,
}: StyledMenuItemProps) {
  return (
    <button
      type="button"
      className={`ic-context-menu-item ${className}`.trim()}
      role="menuitem"
      onClick={onClick}
    >
      {children}
    </button>
  );
}

export function DeleteButton({
  className = "",
  children,
  onClick,
}: StyledMenuItemProps) {
  return (
    <StyledMenuItem
      className={`ic-context-menu-item--delete ${className}`.trim()}
      onClick={onClick}
    >
      {children}
    </StyledMenuItem>
  );
}
