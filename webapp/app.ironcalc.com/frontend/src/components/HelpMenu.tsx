import styled from "@emotion/styled";
import { Menu, MenuItem } from "@mui/material";
import { BookOpen, Keyboard } from "lucide-react";
import { useRef, useState } from "react";

export function HelpMenu() {
  const [isMenuOpen, setMenuOpen] = useState(false);
  const anchorElement = useRef<HTMLButtonElement>(null);

  const handleClick = () => {
    setMenuOpen(true);
  };

  const handleClose = () => {
    setMenuOpen(false);
  };

  return (
    <div>
      <HelpButton
        type="button"
        ref={anchorElement}
        id="help-button"
        aria-controls={isMenuOpen ? "help-menu" : undefined}
        aria-haspopup="true"
        onClick={handleClick}
        $isActive={isMenuOpen}
      >
        Help
      </HelpButton>
      <Menu
        id="help-menu"
        anchorEl={anchorElement.current}
        open={isMenuOpen}
        onClose={handleClose}
        autoFocus={false}
        disableRestoreFocus={true}
        sx={{
          "& .MuiPaper-root": {
            borderRadius: "8px",
            padding: "4px 0px",
            transform: "translate(-4px, 4px)",
          },
          "& .MuiList-root": { padding: "0" },
          transform: "translate(-4px, 4px)",
        }}
        slotProps={{
          list: {
            "aria-labelledby": "help-button",
            tabIndex: -1,
          },
        }}
      >
        <MenuItemWrapper
          onClick={() => {
            handleClose();
            window.open(
              "https://docs.ironcalc.com/web-application/about.html",
              "_blank",
              "noopener,noreferrer",
            );
          }}
        >
          <StyledIcon>
            <BookOpen />
          </StyledIcon>
          <MenuItemText>Documentation</MenuItemText>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={() => {
            handleClose();
            window.open(
              "https://docs.ironcalc.com/features/keyboard-shortcuts.html",
              "_blank",
              "noopener,noreferrer",
            );
          }}
        >
          <StyledIcon>
            <Keyboard />
          </StyledIcon>
          <MenuItemText>Keyboard Shortcuts</MenuItemText>
        </MenuItemWrapper>
      </Menu>
    </div>
  );
}

const HelpButton = styled.button<{ $isActive?: boolean }>`
  display: flex;
  align-items: center;
  font-size: 12px;
  font-family: Inter;
  padding: 8px;
  border-radius: 4px;
  cursor: pointer;
  background-color: ${(props) => (props.$isActive ? "#e6e6e6" : "transparent")};
  border: none;
  background: none;
  &:hover {
    background-color: #f2f2f2;
  }
`;

const MenuItemWrapper = styled(MenuItem)`
  display: flex;
  align-items: center;
  justify-content: flex-start;
  font-size: 14px;
  width: calc(100% - 8px);
  min-width: 172px;
  margin: 0px 4px;
  border-radius: 4px;
  padding: 8px;
  height: 32px;
`;

const StyledIcon = styled.div`
  display: flex;
  align-items: center;
  svg {
    width: 16px;
    height: 100%;
    color: #757575;
    padding-right: 10px;
  }
`;

const MenuItemText = styled.div`
  color: #000;
  font-size: 12px;
`;
