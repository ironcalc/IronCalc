import styled from "@emotion/styled";
import { Menu } from "@mui/material";
import { BookOpen, Keyboard } from "lucide-react";
import { useRef, useState } from "react";
import { MenuItemWrapper } from "./FileMenu";

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
          <BookOpen />
          Documentation
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
          <Keyboard />
          Keyboard Shortcuts
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
  &:hover {
    background-color: #f2f2f2;
  }
`;
