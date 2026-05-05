import { useCallback, useRef, useState } from "react";

export function useLanguageSubmenu() {
  const [isOpen, setIsOpen] = useState(false);
  const [menuStyle, setMenuStyle] = useState<{ left?: number; top?: number }>(
    {},
  );
  const anchorElement = useRef<HTMLButtonElement>(null);
  const closeTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleMouseEnter = useCallback(() => {
    if (closeTimer.current) {
      clearTimeout(closeTimer.current);
      closeTimer.current = null;
    }
    const rect = anchorElement.current?.getBoundingClientRect();
    if (rect) setMenuStyle({ left: rect.right, top: rect.top - 4 });
    setIsOpen(true);
  }, []);

  const handleMouseLeave = useCallback(() => {
    closeTimer.current = setTimeout(() => setIsOpen(false), 80);
  }, []);

  const close = useCallback(() => setIsOpen(false), []);

  return { isOpen, menuStyle, anchorElement, handleMouseEnter, handleMouseLeave, close };
}
