import React, { useState } from "react";
import styled from "@emotion/styled";
import { IronCalcLogo } from "@ironcalc/workbook";
import { Avatar, Drawer, IconButton, MenuItem, Menu } from "@mui/material";
import { EllipsisVertical, HardDrive, Plus, Trash2 } from "lucide-react";
import DeleteWorkbookDialog from "./DeleteWorkbookDialog";
import UserMenu from "./UserMenu";

interface LeftDrawerProps {
  open: boolean;
  onClose: () => void;
  newModel: () => void;
  setModel: (key: string) => void;
  models: { [key: string]: string };
  selectedUuid: string | null;
  setDeleteDialogOpen: (open: boolean) => void;
}

const LeftDrawer: React.FC<LeftDrawerProps> = ({
  open,
  onClose,
  newModel,
  setModel,
  models,
  selectedUuid,
  setDeleteDialogOpen,
}) => {
  const [hoveredUuid, setHoveredUuid] = useState<string | null>(null);
  const [menuAnchorEl, setMenuAnchorEl] = useState<null | HTMLElement>(null);
  const [selectedWorkbookUuid, setSelectedWorkbookUuid] = useState<
    string | null
  >(null);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [userMenuAnchorEl, setUserMenuAnchorEl] = useState<null | HTMLElement>(
    null
  );

  const handleMenuOpen = (
    event: React.MouseEvent<HTMLButtonElement>,
    uuid: string
  ) => {
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

  const elements = Object.keys(models).map((uuid) => (
    <MenuItemWrapper
      key={uuid}
      onClick={() => {
        setModel(uuid);
      }}
      selected={uuid === selectedUuid}
      onMouseEnter={() => setHoveredUuid(uuid)}
      onMouseLeave={() => setHoveredUuid(null)}
      disableRipple
    >
      <StorageIndicator>
        <HardDrive />
      </StorageIndicator>
      <MenuItemText
        style={{
          maxWidth: "240px",
          overflow: "hidden",
          textOverflow: "ellipsis",
        }}
      >
        {models[uuid]}
      </MenuItemText>
      {hoveredUuid === uuid && (
        <EllipsisButton onClick={(e) => handleMenuOpen(e, uuid)}>
          <EllipsisVertical />
        </EllipsisButton>
      )}
    </MenuItemWrapper>
  ));

  return (
    <DrawerWrapper
      variant="persistent"
      anchor="left"
      open={open}
      onClose={onClose}
      sx={{ width: 264, height: "100%" }}
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
        <Menu
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
          sx={{
            "& .MuiPaper-root": {
              minWidth: "auto",
              boxShadow: "0px 1px 3px rgba(0, 0, 0, 0.1)",
            },
          }}
        >
          <MenuItem
            onClick={() => {
              handleMenuClose();
              setIsDeleteDialogOpen(true);
            }}
            sx={{ gap: 1, fontSize: 14 }}
          >
            <Trash2 size={16} />
            Delete workbook
          </MenuItem>
        </Menu>
        {isDeleteDialogOpen && (
          <DeleteWorkbookDialog
            workbookName={
              selectedWorkbookUuid ? models[selectedWorkbookUuid] : ""
            }
            onClose={() => setIsDeleteDialogOpen(false)}
            onConfirm={() => {
              // Handle delete confirmation
              console.log("Delete workbook:", selectedWorkbookUuid);
            }}
          />
        )}
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
    stroke: #757575;
  }
`;

const EllipsisButton = styled(IconButton)`
  background: none;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  height: 24px;
  width: 24px;
  border-radius: 8px;
  color: #333333;
  stroke-width: 2px;
  opacity: 0;
  transition: opacity 0.3s;
  &:hover {
    background: none;
    opacity: 1;
  }
`;

const MenuItemWrapper = styled(MenuItem)<{ selected: boolean }>`
  display: flex;
  gap: 8px;
  justify-content: flex-start;
  font-size: 14px;
  width: 100%;
  min-width: 172px;
  border-radius: 8px;
  padding: 8px 4px 8px 8px;
  height: 32px;
  transition: gap 0.5s;
  background-color: ${({ selected }) =>
    selected ? "#e0e0e0 !important" : "transparent"};
  &:hover {
    gap: 12px;
    transition: gap 0.1s;
  }
`;

const MenuItemText = styled("div")`
  color: #000;
  font-size: 12px;
  width: 100%;
`;

const DrawerFooter = styled("div")`
  display: flex;
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
