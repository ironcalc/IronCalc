import { styled } from "@mui/material/styles";
import type React from "react";
import {
  createContext,
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { createPortal } from "react-dom";

export const SubmenuContext = createContext<{
  openSubmenuAnchor: HTMLElement | null;
  setOpenSubmenuAnchor: (el: HTMLElement | null) => void;
}>({
  openSubmenuAnchor: null,
  setOpenSubmenuAnchor: () => {},
});

export type PopperPlacementType =
  | "bottom"
  | "bottom-end"
  | "bottom-start"
  | "left"
  | "left-end"
  | "left-start"
  | "right"
  | "right-end"
  | "right-start"
  | "top"
  | "top-end"
  | "top-start";

export type MenuProps = {
  open: boolean;
  onClose: () => void;
  anchorEl: React.RefObject<HTMLElement | null>;
  placement: PopperPlacementType;
  children: React.ReactNode;
  offset: [number, number];
};

const DEFAULT_OFFSET: [number, number] = [-4, 4];

/** Right-aligned placements (menu anchored to end/right); flip skid so offset looks correct. */
function isRightAlignedPlacement(placement: PopperPlacementType): boolean {
  return placement.includes("-end") || placement.startsWith("right");
}

/** Used by onClickAway to ignore clicks inside this menu or any submenu (submenus render in separate Poppers/portals). */
export const MENU_PANEL_DATA_ATTR = "data-menu-panel";

function getPlacementParts(placement: PopperPlacementType): {
  base: "top" | "bottom" | "left" | "right";
  align: "start" | "end" | "center";
} {
  const [base, align] = placement.split("-");
  return {
    base: (base ?? "bottom") as "top" | "bottom" | "left" | "right",
    align: (align === "start" ? "start" : align === "end" ? "end" : "center") as
      | "start"
      | "end"
      | "center",
  };
}

function computePosition(
  anchorRect: DOMRect,
  panelWidth: number,
  panelHeight: number,
  placement: PopperPlacementType,
  [skidding, distance]: [number, number],
): { top: number; left: number } {
  const { base, align } = getPlacementParts(placement);
  let top = 0;
  let left = 0;

  const startX = anchorRect.left;
  const endX = anchorRect.right;
  const centerX = anchorRect.left + anchorRect.width / 2;
  const startY = anchorRect.top;
  const endY = anchorRect.bottom;
  const centerY = anchorRect.top + anchorRect.height / 2;

  if (base === "bottom") {
    top = endY + distance;
    left =
      align === "start"
        ? startX + skidding
        : align === "end"
          ? endX - panelWidth + skidding
          : centerX - panelWidth / 2 + skidding;
  } else if (base === "top") {
    top = startY - panelHeight - distance;
    left =
      align === "start"
        ? startX + skidding
        : align === "end"
          ? endX - panelWidth + skidding
          : centerX - panelWidth / 2 + skidding;
  } else if (base === "right") {
    left = endX + distance;
    top =
      align === "start"
        ? startY + skidding
        : align === "end"
          ? endY - panelHeight + skidding
          : centerY - panelHeight / 2 + skidding;
  } else {
    left = startX - panelWidth - distance;
    top =
      align === "start"
        ? startY + skidding
        : align === "end"
          ? endY - panelHeight + skidding
          : centerY - panelHeight / 2 + skidding;
  }
  return { top, left };
}

export function Menu({
  open,
  onClose,
  anchorEl,
  placement,
  children,
  offset = DEFAULT_OFFSET,
}: MenuProps) {
  const [openSubmenuAnchor, setOpenSubmenuAnchor] =
    useState<HTMLElement | null>(null);
  const [position, setPosition] = useState({ top: -10000, left: -10000 });
  const panelRef = useRef<HTMLDivElement | null>(null);

  const effectiveOffset: [number, number] = useMemo(
    () =>
      isRightAlignedPlacement(placement) ? [-offset[0], offset[1]] : offset,
    [placement, offset],
  );

  const updatePosition = useCallback(() => {
    const anchor = anchorEl.current;
    const panel = panelRef.current;
    if (!anchor || !panel) return;
    const anchorRect = anchor.getBoundingClientRect();
    const panelRect = panel.getBoundingClientRect();
    setPosition(
      computePosition(
        anchorRect,
        panelRect.width,
        panelRect.height,
        placement,
        effectiveOffset,
      ),
    );
  }, [anchorEl, placement, effectiveOffset]);

  useEffect(() => {
    if (!open) setPosition({ top: -10000, left: -10000 });
  }, [open]);

  useLayoutEffect(() => {
    if (!open) return;
    updatePosition();
    const anchor = anchorEl.current;
    if (!anchor) return;
    const resizeObserver = new ResizeObserver(updatePosition);
    resizeObserver.observe(anchor);
    const onScrollOrResize = () => updatePosition();
    window.addEventListener("scroll", onScrollOrResize, true);
    window.addEventListener("resize", onScrollOrResize);
    return () => {
      resizeObserver.disconnect();
      window.removeEventListener("scroll", onScrollOrResize, true);
      window.removeEventListener("resize", onScrollOrResize);
    };
  }, [open, updatePosition, anchorEl]);

  useEffect(() => {
    if (!open) return;
    const handleClickAway = (event: MouseEvent | TouchEvent) => {
      const target = event.target as Node;
      if (
        target instanceof Element &&
        target.closest(`[${MENU_PANEL_DATA_ATTR}]`)
      ) {
        return;
      }
      if (anchorEl.current?.contains(target)) return;
      onClose();
    };
    document.addEventListener("mousedown", handleClickAway);
    document.addEventListener("touchstart", handleClickAway);
    return () => {
      document.removeEventListener("mousedown", handleClickAway);
      document.removeEventListener("touchstart", handleClickAway);
    };
  }, [open, onClose, anchorEl]);

  if (!open) {
    return (
      <SubmenuContext.Provider
        value={{ openSubmenuAnchor, setOpenSubmenuAnchor }}
      >
        {null}
      </SubmenuContext.Provider>
    );
  }

  const panel = (
    <StyledPositionedWrapper
      ref={panelRef}
      style={{ top: position.top, left: position.left }}
    >
      <MenuPanel {...{ [MENU_PANEL_DATA_ATTR]: "" }}>{children}</MenuPanel>
    </StyledPositionedWrapper>
  );

  return (
    <SubmenuContext.Provider
      value={{ openSubmenuAnchor, setOpenSubmenuAnchor }}
    >
      {createPortal(panel, document.body)}
    </SubmenuContext.Provider>
  );
}

const StyledPositionedWrapper = styled("div")`
  position: fixed;
  z-index: 1300;
  pointer-events: auto;
`;

export const MenuPanel = styled("div")`
  border-radius: 8px;
  padding: 4px 0;
  min-width: 172px;
  box-shadow: 1px 2px 8px rgba(139, 143, 173, 0.5);
  background: ${({ theme }) => theme.palette.background.default};
  font-family: ${({ theme }) => theme.typography.fontFamily};
  font-size: 12px;
  overflow: hidden;
`;

export function MenuDivider() {
  return <Divider />;
}

const Divider = styled("div")`
  width: 100%;
  margin: 4px auto;
  border-top: 1px solid ${({ theme }) => theme.palette.divider};
`;
