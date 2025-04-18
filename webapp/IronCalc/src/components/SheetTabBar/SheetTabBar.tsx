import { styled } from "@mui/material";
import { Menu, Plus } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { theme } from "../../theme";
import { StyledButton } from "../Toolbar/Toolbar";
import { NAVIGATION_HEIGHT } from "../constants";
import type { WorkbookState } from "../workbookState";
import SheetListMenu from "./SheetListMenu";
import SheetTab from "./SheetTab";
import type { SheetOptions } from "./types";

export interface SheetTabBarProps {
  sheets: SheetOptions[];
  selectedIndex: number;
  workbookState: WorkbookState;
  onSheetSelected: (index: number) => void;
  onAddBlankSheet: () => void;
  onSheetColorChanged: (hex: string) => void;
  onSheetRenamed: (name: string) => void;
  onSheetDeleted: () => void;
  onHideSheet: () => void;
}

function SheetTabBar(props: SheetTabBarProps) {
  const { t } = useTranslation();
  const { workbookState, onSheetSelected, sheets, selectedIndex } = props;
  const [anchorEl, setAnchorEl] = useState<null | HTMLButtonElement>(null);
  const open = Boolean(anchorEl);
  const handleClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    setAnchorEl(event.currentTarget);
  };
  const handleClose = () => {
    setAnchorEl(null);
  };

  const nonHidenSheets = sheets
    .map((s, index) => {
      return {
        state: s.state,
        index,
        name: s.name,
        color: s.color,
        sheetId: s.sheetId,
      };
    })
    .filter((s) => s.state === "visible");

  return (
    <Container>
      <LeftButtonsContainer>
        <StyledButton
          title={t("navigation.add_sheet")}
          $pressed={false}
          onClick={props.onAddBlankSheet}
        >
          <Plus />
        </StyledButton>
        <StyledButton
          onClick={handleClick}
          title={t("navigation.sheet_list")}
          $pressed={false}
        >
          <Menu />
        </StyledButton>
      </LeftButtonsContainer>
      <VerticalDivider />
      <Sheets>
        <SheetInner>
          {nonHidenSheets.map((tab) => (
            <SheetTab
              key={tab.sheetId}
              name={tab.name}
              color={tab.color}
              selected={tab.index === selectedIndex}
              onSelected={() => onSheetSelected(tab.index)}
              onColorChanged={(hex: string): void => {
                props.onSheetColorChanged(hex);
              }}
              onRenamed={(name: string): void => {
                props.onSheetRenamed(name);
              }}
              canDelete={nonHidenSheets.length > 1}
              onDeleted={(): void => {
                props.onSheetDeleted();
              }}
              onHideSheet={props.onHideSheet}
              workbookState={workbookState}
            />
          ))}
        </SheetInner>
      </Sheets>
      <Advert href="https://www.ironcalc.com" target="_blank">
        ironcalc.com
      </Advert>
      <SheetListMenu
        anchorEl={anchorEl}
        open={open}
        onClose={handleClose}
        sheetOptionsList={sheets}
        onSheetSelected={(index) => {
          onSheetSelected(index);
          handleClose();
        }}
        selectedIndex={selectedIndex}
      />
    </Container>
  );
}

// Note I have to specify the font-family in every component that can be considered stand-alone
const Container = styled("div")`
  display: flex;
  flex-direction: row;
  position: absolute;
  bottom: 0px;
  left: 0px;
  right: 0px;
  display: flex;
  height: ${NAVIGATION_HEIGHT}px;
  align-items: center;
  padding: 0px 12px;
  font-family: Inter;
  background-color: ${theme.palette.common.white};
  border-top: 1px solid ${theme.palette.grey["300"]};
  @media (max-width: 769px) {
    padding-right: 0px;
    padding-left: 8px;
  }
`;

const Sheets = styled("div")`
  flex-grow: 2;
  overflow: hidden;
  overflow-x: auto;
  scrollbar-width: none;
  padding-left: 12px;
  display: flex;
  flex-direction: row;
`;

const SheetInner = styled("div")`
  display: flex;
`;

const Advert = styled("a")`
  display: flex;
  align-items: center;
  color: ${theme.palette.primary.main};
  padding: 0px 0px 0px 12px;
  font-size: 12px;
  text-decoration: none;
  border-left: 1px solid ${theme.palette.grey["300"]};
  transition: color 0.2s ease-in-out;
  &:hover {
    text-decoration: underline;
  }
  @media (max-width: 769px) {
    display: none;
  }
`;

const LeftButtonsContainer = styled("div")`
  display: flex;
  flex-direction: row;
  gap: 4px;
  padding-right: 12px;
  @media (max-width: 769px) {
    padding-right: 8px;
  }
`;

const VerticalDivider = styled("div")`
  height: 100%;
  width: 0px;
  @media (max-width: 769px) {
    border-right: 1px solid ${theme.palette.grey["200"]};
  }
`;

export default SheetTabBar;
