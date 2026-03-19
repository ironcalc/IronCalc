import type { Model } from "@ironcalc/wasm";
import { Menu, MenuItem, styled } from "@mui/material";
import { Tag } from "lucide-react";
import { useCallback, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { parseRangeInSheet } from "../Editor/util";

type FormulaBarMenuProps = {
  children: React.ReactNode;
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
const StyledMenu = styled(Menu)({
  top: 4,
  minWidth: 260,
  maxWidth: 460,

  "& .MuiPaper-root": {
    borderRadius: 8,
    padding: "4px 0px",
    marginLeft: -4,
  },

  "& .MuiList-root": {
    padding: 0,
  },
});

const MenuItemWrapper = styled(MenuItem)(({ theme }) => ({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  fontSize: 12,
  gap: 8,
  width: "calc(100% - 8px)",
  minWidth: 172,
  margin: "0px 4px",
  borderRadius: 4,
  padding: 8,
  height: 32,

  "& svg": {
    width: 12,
    height: 12,
    flexShrink: 0,
    color: theme.palette.grey[600],
  },
}));

const ChildrenWrapper = styled("div")({
  display: "flex",
});

const MenuDivider = styled("div")(({ theme }) => ({
  width: "100%",
  margin: "auto",
  marginTop: 4,
  marginBottom: 4,
  borderTop: `1px solid ${theme.palette.grey[200]}`,
}));

const MenuItemText = styled("div")(({ theme }) => ({
  flex: 1,
  minWidth: 0,
  color: theme.palette.common.black,
  overflow: "hidden",
  textOverflow: "ellipsis",
  whiteSpace: "nowrap",
}));

const MenuItemExample = styled("div")(({ theme }) => ({
  color: theme.palette.grey[400],
  marginLeft: 12,
}));

export default FormulaBarMenu;
