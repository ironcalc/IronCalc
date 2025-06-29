import styled from "@emotion/styled";
import { IronCalcLogo } from "@ironcalc/workbook";
import { Avatar, Drawer, IconButton, Menu, MenuItem } from "@mui/material";
import {
  EllipsisVertical,
  FileDown,
  FileSpreadsheet,
  Plus,
  Trash2,
} from "lucide-react";
import type React from "react";
import { useState } from "react";
import UserMenu from "../UserMenu";

interface LeftDrawerProps {
  open: boolean;
  onClose: () => void;
  newModel: () => void;
  setModel: (key: string) => void;
  models: { [key: string]: string };
  selectedUuid: string | null;
}

const LeftDrawer: React.FC<LeftDrawerProps> = ({
  open,
  onClose,
  newModel,
  setModel,
  models,
  selectedUuid,
}) => {
  const [menuAnchorEl, setMenuAnchorEl] = useState<null | HTMLElement>(null);
  const [selectedWorkbookUuid, setSelectedWorkbookUuid] = useState<
    string | null
  >(null);
  const [userMenuAnchorEl, setUserMenuAnchorEl] = useState<null | HTMLElement>(
    null,
  );

  const handleMenuOpen = (
    event: React.MouseEvent<HTMLButtonElement>,
    uuid: string,
  ) => {
    console.log("Menu open", uuid);
    event.stopPropagation();
    setSelectedWorkbookUuid(uuid);
    setMenuAnchorEl(event.currentTarget);
  };

  const handleMenuClose = () => {
    setMenuAnchorEl(null);
    setSelectedWorkbookUuid(null);
  };

  const handleUserMenuOpen = (event: React.MouseEvent<HTMLElement>) => {
    setUserMenuAnchorEl(event.currentTarget);
  };

  const handleUserMenuClose = () => {
    setUserMenuAnchorEl(null);
  };

  const elements = Object.keys(models)
    .reverse()
    .map((uuid) => {
      const isMenuOpen = menuAnchorEl !== null && selectedWorkbookUuid === uuid;
      return (
        <WorkbookListItem
          key={uuid}
          onClick={() => {
            setModel(uuid);
          }}
          selected={uuid === selectedUuid}
          disableRipple
        >
          <StorageIndicator>
            <FileSpreadsheet />
          </StorageIndicator>
          <WorkbookListText>{models[uuid]}</WorkbookListText>
          <EllipsisButton
            onClick={(e) => handleMenuOpen(e, uuid)}
            disableRipple
            isOpen={isMenuOpen}
          >
            <EllipsisVertical />
          </EllipsisButton>

          <StyledMenu
            anchorEl={menuAnchorEl}
            open={Boolean(menuAnchorEl)}
            onClose={handleMenuClose}
            MenuListProps={{
              dense: true,
            }}
            anchorOrigin={{
              vertical: "bottom",
              horizontal: "right",
            }}
            transformOrigin={{
              vertical: "top",
              horizontal: "right",
            }}
            disablePortal
          >
            <MenuItemWrapper
              onClick={() => {
                handleMenuClose();
              }}
              disableRipple
            >
              <FileDown />
              Download (.xlsx)
            </MenuItemWrapper>
            <MenuDivider />
            <MenuItemWrapper
              selected={false}
              onClick={() => {
                handleMenuClose();
              }}
              disableRipple
            >
              <Trash2 size={16} />
              Delete workbook
            </MenuItemWrapper>
          </StyledMenu>
        </WorkbookListItem>
      );
    });

  return (
    <DrawerWrapper
      variant="persistent"
      anchor="left"
      open={open}
      onClose={onClose}
    >
      <DrawerHeader>
        <StyledDesktopLogo />
        <AddButton
          onClick={() => {
            newModel();
          }}
        >
          <PlusIcon />
        </AddButton>
      </DrawerHeader>
      <DrawerContent>
        <DrawerContentTitle>Your workbooks</DrawerContentTitle>
        {elements}
      </DrawerContent>
      <DrawerFooter>
        <UserWrapper
          disableRipple
          onClick={handleUserMenuOpen}
          selected={Boolean(userMenuAnchorEl)}
        >
          <StyledAvatar
            alt="Nikola Tesla"
            src="/path/to/avatar.jpg"
            sx={{ bgcolor: "#f2994a", width: 24, height: 24 }}
          />
          <Username>Nikola Tesla</Username>
        </UserWrapper>
        <UserMenu
          anchorEl={userMenuAnchorEl}
          onClose={handleUserMenuClose}
          onPreferences={() => {
            console.log("Preferences clicked");
            handleUserMenuClose();
          }}
          onLogout={() => {
            console.log("Logout clicked");
            handleUserMenuClose();
          }}
        />
      </DrawerFooter>
    </DrawerWrapper>
  );
};

const DrawerWrapper = styled(Drawer)`
  width: 264px;
  height: 100%;
  flex-shrink: 0;
  font-family: "Inter", sans-serif;

  .MuiDrawer-paper {
    width: 264px;
    background-color: #f5f5f5;
    overflow: hidden;
    border-right: 1px solid #e0e0e0;
  }
`;

const DrawerHeader = styled("div")`
  display: flex;
  align-items: center;
  padding: 12px 8px 12px 16px;
  justify-content: space-between;
  max-height: 60px;
  min-height: 60px;
  border-bottom: 1px solid #e0e0e0;
  box-sizing: border-box;
`;

const StyledDesktopLogo = styled(IronCalcLogo)`
  width: 120px;
  height: 28px;
`;

const AddButton = styled(IconButton)`
  background: none;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px;
  height: 32px;
  width: 32px;
  border-radius: 4px;
  margin-left: 10px;
  color: #333333;
  stroke-width: 2px;
  &:hover {
    background-color: #e0e0e0;
  }
`;

const PlusIcon = styled(Plus)`
  width: 16px;
  height: 16px;
`;

const DrawerContent = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 16px 12px;
  height: 100%;
  overflow: scroll;
  font-size: 12px;
`;

const DrawerContentTitle = styled("div")`
  font-weight: 600;
  color: #9e9e9e;
  margin-bottom: 8px;
  padding: 0px 8px;
`;

const StorageIndicator = styled("div")`
  height: 16px;
  width: 16px;
  svg {
    height: 16px;
    width: 16px;
    stroke: #9e9e9e;
  }
`;

const EllipsisButton = styled(IconButton)<{ isOpen: boolean }>`
  background: none;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  height: 24px;
  width: 24px;
  border-radius: 4px;
  color: #333333;
  stroke-width: 2px;
  background-color: ${({ isOpen }) => (isOpen ? "#E0E0E0" : "none")};
  opacity: ${({ isOpen }) => (isOpen ? "1" : "0.5")};
  transition: opacity 0.3s, background-color 0.3s;
  &:hover {
    background: none;
    opacity: 1;
  }
  &:active {
    background: #bdbdbd;
    opacity: 1;
  }
`;

const WorkbookListItem = styled(MenuItem)<{ selected: boolean }>`
  display: flex;
  gap: 8px;
  justify-content: flex-start;
  font-size: 14px;
  width: 100%;
  min-width: 172px;
  border-radius: 8px;
  padding: 8px 4px 8px 8px;
  height: 32px;
  min-height: 32px;
  transition: gap 0.5s;
  background-color: ${({ selected }) =>
    selected ? "#e0e0e0 !important" : "transparent"};
`;

const WorkbookListText = styled("div")`
  color: #000;
  font-size: 12px;
  width: 100%;
  max-width: 240px;
  overflow: hidden;
  text-overflow: ellipsis;
`;

const StyledMenu = styled(Menu)`
  .MuiPaper-root {
    border-radius: 8px;
    padding: 4px 0px;
    box-shadow: 0px 2px 4px rgba(0, 0, 0, 0.01);
  },
  .MuiList-root {
    padding: 0;
  },
`;

const MenuDivider = styled("div")`
  width: 100%;
  margin: auto;
  margin-top: 4px;
  margin-bottom: 4px;
  border-top: 1px solid #eeeeee;
`;

const MenuItemWrapper = styled(MenuItem)`
  display: flex;
  justify-content: flex-start;
  font-size: 12px;
  width: calc(100% - 8px);
  min-width: 140px;
  margin: 0px 4px;
  border-radius: 4px;
  padding: 8px;
  height: 32px;
  gap: 8px;
  svg {
    width: 16px;
    height: 16px;
  }
`;

const DrawerFooter = styled("div")`
  display: none;
  align-items: center;
  padding: 12px;
  justify-content: space-between;
  max-height: 60px;
  height: 60px;
  border-top: 1px solid #e0e0e0;
  box-sizing: border-box;
`;

const UserWrapper = styled(MenuItem)<{ selected: boolean }>`
  display: flex;
  align-items: center;
  gap: 8px;
  flex-grow: 1;
  padding: 8px;
  border-radius: 8px;
  max-width: 100%;
  background-color: ${({ selected }) =>
    selected ? "#e0e0e0 !important" : "transparent"};
`;

const StyledAvatar = styled(Avatar)`
  font-size: 14px;
`;

const Username = styled("div")`
  font-size: 12px;
  flex-grow: 1;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
`;

export default LeftDrawer;
