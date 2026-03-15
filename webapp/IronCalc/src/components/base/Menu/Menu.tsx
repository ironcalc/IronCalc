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
export const MENU_PANEL_DATA_ATTR = "data-menu-panel";

function computePosition(
  anchorRect: DOMRect,
  panelWidth: number,
  panelHeight: number,
  placement: PopperPlacementType,
  [skidding, distance]: [number, number],
): { top: number; left: number } {
  // ✂️ Inlined getPlacementParts
  const [basePart, alignPart] = placement.split("-");
  const base = (basePart ?? "bottom") as "top" | "bottom" | "left" | "right";
  const align =
    alignPart === "start" ? "start" : alignPart === "end" ? "end" : "center";

  const {
    left: aL,
    right: aR,
    top: aT,
    bottom: aB,
    width: aW,
    height: aH,
  } = anchorRect;

  if (base === "bottom" || base === "top") {
    const top = base === "bottom" ? aB + distance : aT - panelHeight - distance;
    const left =
      align === "start"
        ? aL + skidding
        : align === "end"
          ? aR - panelWidth + skidding
          : aL + aW / 2 - panelWidth / 2 + skidding;
    return { top, left };
  } else {
    const left = base === "right" ? aR + distance : aL - panelWidth - distance;
    const top =
      align === "start"
        ? aT + skidding
        : align === "end"
          ? aB - panelHeight + skidding
          : aT + aH / 2 - panelHeight / 2 + skidding;
    return { top, left };
  }
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
      placement.includes("-end") || placement.startsWith("right")
        ? [-offset[0], offset[1]]
        : offset,
    [placement, offset[0], offset[1]],
  );

  const updatePosition = useCallback(() => {
    const anchor = anchorEl.current;
    const panel = panelRef.current;
    if (!anchor || !panel) return;
    const anchorRect = anchor.getBoundingClientRect();
    const { width, height } = panel.getBoundingClientRect();
    setPosition(
      computePosition(anchorRect, width, height, placement, effectiveOffset),
    );
  }, [anchorEl, placement, effectiveOffset]);

  useEffect(() => {
    if (!open) {
      setPosition({ top: -10000, left: -10000 });
      setOpenSubmenuAnchor(null);
    }
  }, [open]);

  useLayoutEffect(() => {
    if (!open) return;
    updatePosition();
    const anchor = anchorEl.current;
    if (!anchor) return;
    const resizeObserver = new ResizeObserver(updatePosition);
    resizeObserver.observe(anchor);
    // ✂️ Pass updatePosition directly, no wrapper needed
    window.addEventListener("scroll", updatePosition, true);
    window.addEventListener("resize", updatePosition);
    return () => {
      resizeObserver.disconnect();
      window.removeEventListener("scroll", updatePosition, true);
      window.removeEventListener("resize", updatePosition);
    };
  }, [open, updatePosition, anchorEl]);

  useEffect(() => {
    if (!open) return;
    const handleClickAway = (event: MouseEvent | TouchEvent) => {
      const target = event.target as Node;
      const el = target instanceof Element ? target : target.parentElement;
      if (el?.closest(`[${MENU_PANEL_DATA_ATTR}]`)) return;
      if (el && anchorEl.current?.contains(el)) return;
      onClose();
    };
    document.addEventListener("mousedown", handleClickAway);
    document.addEventListener("touchstart", handleClickAway);
    return () => {
      document.removeEventListener("mousedown", handleClickAway);
      document.removeEventListener("touchstart", handleClickAway);
    };
  }, [open, onClose, anchorEl]);

  // ✂️ Single return — context always wraps, portal only rendered when open
  return (
    <SubmenuContext.Provider
      value={{ openSubmenuAnchor, setOpenSubmenuAnchor }}
    >
      {open &&
        createPortal(
          <StyledPositionedWrapper ref={panelRef} style={position}>
            <MenuPanel {...{ [MENU_PANEL_DATA_ATTR]: "" }}>
              {children}
            </MenuPanel>
          </StyledPositionedWrapper>,
          document.body,
        )}
    </SubmenuContext.Provider>
  );
}

// Styled components unchanged
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
