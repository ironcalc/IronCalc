import type { KeyboardEvent, RefObject } from "react";

export function useMenuKeyDown(
  menuRef: RefObject<HTMLDivElement | null>,
  close: () => void,
  isSubmenu = false,
) {
  function handleMenuKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    const items = Array.from(
      menuRef.current?.querySelectorAll<HTMLButtonElement>(
        ':is([role="menuitem"],[role="menuitemradio"],[role="menuitemcheckbox"]):not([disabled])',
      ) ?? [],
    );

    if (items.length === 0) {
      return;
    }

    const currentIndex = items.indexOf(
      document.activeElement as HTMLButtonElement,
    );

    switch (event.key) {
      case "ArrowDown": {
        event.preventDefault();
        items[(currentIndex + 1) % items.length]?.focus();
        break;
      }

      case "ArrowUp": {
        event.preventDefault();
        items[(currentIndex - 1 + items.length) % items.length]?.focus();
        break;
      }

      case "Home": {
        event.preventDefault();
        items[0]?.focus();
        break;
      }

      case "End": {
        event.preventDefault();
        items[items.length - 1]?.focus();
        break;
      }

      case "ArrowLeft": {
        if (isSubmenu) {
          event.preventDefault();
          close();
        }
        break;
      }

      case "Escape": {
        event.preventDefault();
        close();
        break;
      }

      case "Tab": {
        close();
        break;
      }
    }
  }

  return { handleMenuKeyDown };
}
