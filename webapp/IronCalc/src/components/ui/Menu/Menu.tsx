import ClickAwayListener from "@mui/material/ClickAwayListener";
import Popper, { type PopperPlacementType } from "@mui/material/Popper";
import { styled } from "@mui/material/styles";
import type React from "react";
import { createContext, useState } from "react";

export const SubmenuContext = createContext<{
  openSubmenuAnchor: HTMLElement | null;
  setOpenSubmenuAnchor: (el: HTMLElement | null) => void;
}>({
  openSubmenuAnchor: null,
  setOpenSubmenuAnchor: () => {},
});

export type MenuProps = {
  open: boolean;
  onClose: () => void;
  anchorEl: React.RefObject<HTMLElement | null>;
  placement?: PopperPlacementType;
  children: React.ReactNode;
  offset?: [number, number];
};

const DEFAULT_OFFSET: [number, number] = [-4, 4];

/** Right-aligned placements (menu anchored to end/right); flip skid so offset looks correct. */
function isRightAlignedPlacement(placement: PopperPlacementType): boolean {
  return placement.includes("-end") || placement.startsWith("right");
}

/** Used by onClickAway to ignore clicks inside this menu or any submenu (submenus render in separate Poppers/portals). */
export const MENU_PANEL_DATA_ATTR = "data-menu-panel";

function handleClickAway(onClose: () => void, event: MouseEvent | TouchEvent) {
  const target = event.target as Node;
  if (
    target instanceof Element &&
    target.closest(`[${MENU_PANEL_DATA_ATTR}]`)
  ) {
    return;
  }
  onClose();
}

export function Menu({
  open,
  onClose,
  anchorEl,
  placement = "bottom-start",
  children,
  offset = DEFAULT_OFFSET,
}: MenuProps) {
  const [openSubmenuAnchor, setOpenSubmenuAnchor] =
    useState<HTMLElement | null>(null);

  const [skidding, distance] = offset;
  const effectiveOffset: [number, number] = isRightAlignedPlacement(placement)
    ? [-skidding, distance]
    : offset;

  return (
    <SubmenuContext.Provider
      value={{ openSubmenuAnchor, setOpenSubmenuAnchor }}
    >
      <StyledPopper
        open={open}
        anchorEl={anchorEl.current ?? undefined}
        placement={placement}
        keepMounted={false}
        modifiers={[
          {
            name: "offset",
            options: { offset: effectiveOffset },
          },
        ]}
      >
        <ClickAwayListener
          onClickAway={(event) => handleClickAway(onClose, event)}
        >
          <MenuPanel {...{ [MENU_PANEL_DATA_ATTR]: "" }}>{children}</MenuPanel>
        </ClickAwayListener>
      </StyledPopper>
    </SubmenuContext.Provider>
  );
}

const StyledPopper = styled(Popper)`
  z-index: 1300;
  &[data-popper-placement] {
    pointer-events: auto;
  }
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
