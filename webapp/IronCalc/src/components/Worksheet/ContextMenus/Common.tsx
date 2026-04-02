import {
  type ButtonHTMLAttributes,
  type ReactNode,
  useEffect,
  useRef,
  useState,
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
  const focusedIndexRef = useRef(0);
  const [focusedIndex, setFocusedIndex] = useState(0);

  useEffect(() => {
    if (!open) {
      return;
    }

    setFocusedIndex(0);
  }, [open]);

  useEffect(() => {
    focusedIndexRef.current = focusedIndex;
  }, [focusedIndex]);

  useEffect(() => {
    if (!open) {
      return;
    }

    function handleDocumentKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        onClose();
        return;
      }

      const items = getEnabledMenuItems(menuRef.current);
      if (items.length === 0) {
        return;
      }

      const index = focusedIndexRef.current;

      switch (event.key) {
        case "ArrowDown": {
          event.preventDefault();
          event.stopPropagation();
          setFocusedIndex((index + 1) % items.length);
          break;
        }
        case "Tab": {
          event.preventDefault();
          event.stopPropagation();
          const nextIndex = event.shiftKey
            ? (index - 1 + items.length) % items.length
            : (index + 1) % items.length;
          setFocusedIndex(nextIndex);
          break;
        }
        case "ArrowUp": {
          event.preventDefault();
          event.stopPropagation();
          setFocusedIndex((index - 1 + items.length) % items.length);
          break;
        }
        case "Home": {
          event.preventDefault();
          event.stopPropagation();
          setFocusedIndex(0);
          break;
        }
        case "End": {
          event.preventDefault();
          event.stopPropagation();
          setFocusedIndex(items.length - 1);
          break;
        }
        case "Enter":
        case " ": {
          event.preventDefault();
          event.stopPropagation();
          items[index]?.click();
          break;
        }
      }
    }

    document.addEventListener("keydown", handleDocumentKeyDown, true);
    return () => {
      document.removeEventListener("keydown", handleDocumentKeyDown, true);
    };
  }, [open, onClose]);

  useEffect(() => {
    if (!open) {
      return;
    }

    menuRef.current?.focus();
  }, [open]);

  useEffect(() => {
    if (!open) {
      return;
    }

    const items = getEnabledMenuItems(menuRef.current);
    if (items.length === 0) {
      return;
    }

    const safeIndex = Math.min(focusedIndex, items.length - 1);
    items[safeIndex]?.focus();

    if (safeIndex !== focusedIndex) {
      setFocusedIndex(safeIndex);
    }
  }, [open, focusedIndex]);

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

function getEnabledMenuItems(menu: HTMLDivElement | null): HTMLButtonElement[] {
  if (!menu) {
    return [];
  }

  return Array.from(
    menu.querySelectorAll<HTMLButtonElement>(
      ".ic-context-menu-item:not(:disabled)",
    ),
  );
}

type StyledMenuItemProps = ButtonHTMLAttributes<HTMLButtonElement>;

export function StyledMenuItem({
  className = "",
  children,
  onClick,
  disabled,
}: StyledMenuItemProps) {
  return (
    <button
      type="button"
      className={`ic-context-menu-item ${className}`.trim()}
      role="menuitem"
      onClick={onClick}
      disabled={disabled}
    >
      {children}
    </button>
  );
}

export function DeleteButton({
  className = "",
  children,
  onClick,
  disabled,
}: StyledMenuItemProps) {
  return (
    <StyledMenuItem
      className={`ic-context-menu-item--delete ${className}`.trim()}
      onClick={onClick}
      disabled={disabled}
    >
      {children}
    </StyledMenuItem>
  );
}
