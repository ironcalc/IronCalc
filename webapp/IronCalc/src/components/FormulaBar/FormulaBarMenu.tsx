import type { Model } from "@ironcalc/wasm";
import { Menu, MenuItem, styled } from "@mui/material";
import { Tag } from "lucide-react";
import { useCallback, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { theme } from "../../theme";
import { parseRangeInSheet } from "../Editor/util";

type FormulaBarMenuProps = {
  children: React.ReactNode;
  onChange: (option: string) => void;
  onMenuOpenChange: (isOpen: boolean) => void;
  openDrawer: () => void;
  canEdit: boolean;
  model: Model;
  onUpdate: () => void;
};

const FormulaBarMenu = (properties: FormulaBarMenuProps) => {
  const { t } = useTranslation();
  const [isMenuOpen, setMenuOpen] = useState(false);
  const anchorElement = useRef<HTMLDivElement>(null);

  const handleMenuOpen = useCallback((): void => {
    setMenuOpen(true);
    properties.onMenuOpenChange(true);
  }, [properties.onMenuOpenChange]);

  const handleMenuClose = useCallback((): void => {
    setMenuOpen(false);
    properties.onMenuOpenChange(false);
  }, [properties.onMenuOpenChange]);

  const definedNameList = properties.model.getDefinedNameList();

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
        {definedNameList.length > 0 ? (
          <>
            {definedNameList.map((definedName) => {
              return (
                <MenuItemWrapper
                  key={`${definedName.name}-${definedName.scope}`}
                  disableRipple
                  onClick={() => {
                    // select the area corresponding to the defined name
                    const formula = definedName.formula;
                    const range = parseRangeInSheet(properties.model, formula);
                    if (range) {
                      const [
                        sheetIndex,
                        rowStart,
                        columnStart,
                        rowEnd,
                        columnEnd,
                      ] = range;
                      properties.model.setSelectedSheet(sheetIndex);
                      properties.model.setSelectedCell(rowStart, columnStart);
                      properties.model.setSelectedRange(
                        rowStart,
                        columnStart,
                        rowEnd,
                        columnEnd,
                      );
                    }
                    properties.onUpdate();
                    properties.onChange(definedName.name);
                    handleMenuClose();
                  }}
                >
                  <Tag />
                  <MenuItemText>{definedName.name}</MenuItemText>
                  <MenuItemExample>{definedName.formula}</MenuItemExample>
                </MenuItemWrapper>
              );
            })}
            <MenuDivider />
          </>
        ) : null}
        <MenuItemWrapper
          onClick={() => {
            properties.openDrawer();
            handleMenuClose();
          }}
          disabled={!properties.canEdit}
          disableRipple
        >
          <MenuItemText>{t("formula_bar.manage_named_ranges")}</MenuItemText>
        </MenuItemWrapper>
      </StyledMenu>
    </>
  );
};

const StyledMenu = styled(Menu)`
  top: 4px;
  min-width: 260px;
  max-width: 460px;
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
    flex-shrink: 0;
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
  border-top: 1px solid ${theme.palette.grey[200]};
`;

const MenuItemText = styled("div")`
  flex: 1;
  min-width: 0;
  color: ${theme.palette.common.black};
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
`;

const MenuItemExample = styled("div")`
  color: ${theme.palette.grey[400]};
  margin-left: 12px;
`;

export default FormulaBarMenu;
