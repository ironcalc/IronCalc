import {
  type RefObject,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

export function useMenuAnchor(
  isOpen: boolean,
  onClose: () => void,
  additionalRefs: RefObject<Element | null>[] = [],
) {
  const anchorElement = useRef<HTMLButtonElement>(null);
  const menuElement = useRef<HTMLDivElement>(null);
  const [menuStyle, setMenuStyle] = useState<{ left?: number; top?: number }>(
    {},
  );

  useLayoutEffect(() => {
    if (!isOpen || !anchorElement.current) return;

    const update = () => {
      const rect = anchorElement.current?.getBoundingClientRect();
      if (rect) setMenuStyle({ left: rect.left - 4, top: rect.bottom + 4 });
    };

    update();
    window.addEventListener("resize", update);
    window.addEventListener("scroll", update, true);
    return () => {
      window.removeEventListener("resize", update);
      window.removeEventListener("scroll", update, true);
    };
  }, [isOpen]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: additionalRefs holds stable ref objects; .current is intentionally read inside the handler
  useEffect(() => {
    if (!isOpen) return;

    const onPointerDown = (event: PointerEvent) => {
      const target = event.target as Node | null;
      if (
        anchorElement.current?.contains(target) ||
        menuElement.current?.contains(target) ||
        additionalRefs.some((r) => r.current?.contains(target))
      )
        return;
      onClose();
    };

    document.addEventListener("pointerdown", onPointerDown, true);
    return () =>
      document.removeEventListener("pointerdown", onPointerDown, true);
  }, [isOpen, onClose]);

  return { anchorElement, menuElement, menuStyle };
}
