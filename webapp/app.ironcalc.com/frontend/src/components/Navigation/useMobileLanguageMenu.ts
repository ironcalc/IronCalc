import { useCallback, useRef, useState } from "react";

export function useMobileLanguageMenu() {
  const [isOpen, setIsOpen] = useState(false);
  const [style, setStyle] = useState<{ left?: number; top?: number }>({});
  const anchorRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  const toggle = useCallback(() => {
    if (!isOpen) {
      const rect = anchorRef.current?.getBoundingClientRect();
      if (rect) setStyle({ left: rect.right, top: rect.top - 4 });
    }
    setIsOpen(!isOpen);
  }, [isOpen]);

  const close = useCallback(() => setIsOpen(false), []);

  return { isOpen, style, anchorRef, menuRef, toggle, close };
}
