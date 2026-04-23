import { type CSSProperties, useLayoutEffect, useRef, useState } from "react";

export function useAnchorPosition(
  open: boolean,
  anchor: { x: number; y: number } | undefined,
) {
  const menuRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState<CSSProperties>({});

  useLayoutEffect(() => {
    if (!open || !anchor) {
      return;
    }

    function updatePosition() {
      const menu = menuRef.current;
      if (!menu || !anchor) {
        return;
      }

      const menuWidth = menu.offsetWidth;
      const menuHeight = menu.offsetHeight;
      const viewportWidth = window.innerWidth;
      const viewportHeight = window.innerHeight;
      const margin = 8;

      let left = anchor.x;
      let top = anchor.y;

      // Flip above the anchor when the menu would overflow below and there's room above
      if (
        top + menuHeight > viewportHeight - margin &&
        anchor.y - menuHeight > margin
      ) {
        top = anchor.y - menuHeight;
      }

      if (left + menuWidth > viewportWidth - margin) {
        left = viewportWidth - menuWidth - margin;
      }
      if (left < margin) {
        left = margin;
      }
      if (top + menuHeight > viewportHeight - margin) {
        top = viewportHeight - menuHeight - margin;
      }
      if (top < margin) {
        top = margin;
      }

      setPosition({ top, left });
    }

    updatePosition();

    window.addEventListener("resize", updatePosition);
    window.addEventListener("scroll", updatePosition, true);

    return () => {
      window.removeEventListener("resize", updatePosition);
      window.removeEventListener("scroll", updatePosition, true);
    };
  }, [open, anchor]);

  return { menuRef, position };
}
