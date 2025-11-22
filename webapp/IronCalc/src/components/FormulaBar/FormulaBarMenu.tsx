import { Menu, MenuItem, styled } from "@mui/material";
import { Tag } from "lucide-react";
import { type ComponentProps, useCallback, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { theme } from "../../theme";

type FormulaBarMenuProps = {
  children: React.ReactNode;
  selectedOption?: string;
  onChange: (option: string) => void;
  onExited?: () => void;
  onMenuOpenChange?: (isOpen: boolean) => void;
  anchorOrigin?: ComponentProps<typeof Menu>["anchorOrigin"];
};

const FormulaBarMenu = (properties: FormulaBarMenuProps) => {
  const { t } = useTranslation();
  const [isMenuOpen, setMenuOpen] = useState(false);
  const anchorElement = useRef<HTMLDivElement>(null);

  const handleMenuOpen = useCallback((): void => {
    setMenuOpen(true);
    properties.onMenuOpenChange?.(true);
  }, [properties.onMenuOpenChange]);

  const handleMenuClose = useCallback((): void => {
    setMenuOpen(false);
    properties.onMenuOpenChange?.(false);
  }, [properties.onMenuOpenChange]);

  return (
    <>
      <ChildrenWrapper onClick={handleMenuOpen} ref={anchorElement}>
        {properties.children}
      </ChildrenWrapper>
      <StyledMenu
        open={isMenuOpen}
        onClose={handleMenuClose}
        anchorEl={anchorElement.current}
        marginThreshold={0}
        anchorOrigin={{
          vertical: "bottom",
          horizontal: "left",
        }}
        transformOrigin={{
          vertical: "top",
          horizontal: "left",
        }}
      >
        <MenuItemWrapper disableRipple>
          <Tag />
          <MenuItemText>Range1</MenuItemText>
          <MenuItemExample>$Sheet1!$A$1:$B$2</MenuItemExample>
        </MenuItemWrapper>
        <MenuDivider />
        <MenuItemWrapper disableRipple>
          <MenuItemText>{t("formula_bar.manage_named_ranges")}</MenuItemText>
        </MenuItemWrapper>
      </StyledMenu>
    </>
  );
};

const StyledMenu = styled(Menu)`
  top: 4px;
  min-width: 260px;
  & .MuiPaper-root {
    border-radius: 8px;
    padding: 4px 0px;
    margin-left: -4px;
  }
  & .MuiList-root {
    padding: 0;
  }
`;

const MenuItemWrapper = styled(MenuItem)`
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  gap: 8px;
  width: calc(100% - 8px);
  min-width: 172px;
  margin: 0px 4px;
  border-radius: 4px;
  padding: 8px;
  height: 32px;
  & svg {
    width: 12px;
    height: 12px;
    color: ${theme.palette.grey[600]};
  }
`;

const ChildrenWrapper = styled("div")`
  display: flex;
`;

const MenuDivider = styled("div")`
  width: 100%;
  margin: auto;
  margin-top: 4px;
  margin-bottom: 4px;
  border-top: 1px solid #eeeeee;
`;

const MenuItemText = styled("div")`
  color: #000;
  display: flex;
  align-items: center;
`;

const MenuItemExample = styled("div")`
  color: #bdbdbd;
  margin-left: 20px;
`;

export default FormulaBarMenu;
