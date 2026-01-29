import styled from "@emotion/styled";
import { Popper } from "@mui/material";
import { BookOpen, Keyboard } from "lucide-react";
import { useRef } from "react";
import { useTranslation } from "react-i18next";
import { MenuItemWrapper, MenuPaper } from "./FileMenu";

export function HelpMenu(props: {
  isOpen: boolean;
  onOpen: () => void;
  onClose: () => void;
  onHover: () => void;
}) {
  const anchorElement = useRef<HTMLButtonElement>(null);
  const { t } = useTranslation();

  return (
    <div>
      <HelpButton
        type="button"
        ref={anchorElement}
        id="help-button"
        aria-controls={props.isOpen ? "help-menu" : undefined}
        aria-haspopup="true"
        onClick={props.onOpen}
        onMouseEnter={props.onHover}
        $isActive={props.isOpen}
      >
        {t("file_bar.help_menu.button")}
      </HelpButton>
      <Popper
        id="help-menu"
        anchorEl={anchorElement.current}
        open={props.isOpen}
        placement="bottom-start"
        modifiers={[{ name: "offset", options: { offset: [-4, 4] } }]}
        style={{ zIndex: 1300 }}
      >
        <MenuPaper>
          <MenuItemWrapper
            onClick={() => {
              props.onClose();
              window.open(
                "https://docs.ironcalc.com/web-application/about.html",
                "_blank",
                "noopener,noreferrer",
              );
            }}
          >
            <BookOpen />
            {t("file_bar.help_menu.documentation")}
          </MenuItemWrapper>
          <MenuItemWrapper
            onClick={() => {
              props.onClose();
              window.open(
                "https://docs.ironcalc.com/features/keyboard-shortcuts.html",
                "_blank",
                "noopener,noreferrer",
              );
            }}
          >
            <Keyboard />
            {t("file_bar.help_menu.keyboard_shortcuts")}
          </MenuItemWrapper>
        </MenuPaper>
      </Popper>
    </div>
  );
}

const HelpButton = styled.button<{ $isActive?: boolean }>`
  display: flex;
  align-items: center;
  font-size: 12px;
  font-family: Inter;
  padding: 8px;
  border-radius: 6px;
  cursor: pointer;
  background-color: ${(props) => (props.$isActive ? "#f2f2f2" : "transparent")};
  border: none;
  &:hover {
    background-color: #f2f2f2;
  }
`;
