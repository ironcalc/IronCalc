import styled from "@emotion/styled";
import { Menu, MenuItem } from "@mui/material";
import { LogOut, Settings } from "lucide-react";

interface UserMenuProps {
  anchorEl: null | HTMLElement;
  onClose: () => void;
  onPreferences: () => void;
  onLogout: () => void;
}

const UserMenu: React.FC<UserMenuProps> = ({
  anchorEl,
  onClose,
  onPreferences,
  onLogout,
}) => {
  return (
    <StyledMenu
      anchorEl={anchorEl}
      open={Boolean(anchorEl)}
      onClose={onClose}
      MenuListProps={{
        dense: true,
      }}
      anchorOrigin={{
        vertical: "top",
        horizontal: "left",
      }}
      transformOrigin={{
        vertical: "bottom",
        horizontal: "left",
      }}
    >
      <MenuItemWrapper onClick={onPreferences} sx={{ gap: 1, fontSize: 14 }}>
        <Settings size={16} />
        <MenuItemText>Preferences</MenuItemText>
      </MenuItemWrapper>
      <MenuDivider />
      <MenuItemWrapper onClick={onLogout} sx={{ gap: 1, fontSize: 14 }}>
        <LogOut size={16} />
        <MenuItemText>Log out</MenuItemText>
      </MenuItemWrapper>
    </StyledMenu>
  );
};

const StyledMenu = styled(Menu)`
  & .MuiPaper-root {
    border-radius: 8px;
    padding: 4px 0px;
    margin-top: -4px;
    margin-left: 4px;
  }
  & .MuiList-root {
    padding: 0;
  }
`;

const MenuItemText = styled("div")`
  color: #000;
  font-size: 12px;
  flex-grow: 1;
`;

const MenuItemWrapper = styled(MenuItem)`
  display: flex;
  justify-content: flex-start;
  font-size: 14px;
  width: calc(100% - 8px);
  min-width: 172px;
  margin: 0px 4px;
  border-radius: 4px;
  padding: 8px;
  height: 32px;
  svg {
    width: 16px;
    height: 16px;
  }
`;

const MenuDivider = styled("div")`
  width: 100%;
  margin: auto;
  margin-top: 4px;
  margin-bottom: 4px;
  border-top: 1px solid #eeeeee;
`;

export default UserMenu;
