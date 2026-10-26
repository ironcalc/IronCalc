import {
  Button,
  type IronCalcHandle,
  Menu,
  MenuDivider,
  MenuItem,
  MenuItemWithSubmenu,
  type Model,
} from "@ironcalc/workbook";
import { Columns3, Rows3, Sheet } from "lucide-react";
import { type RefObject, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

export function InsertMenu(props: {
  ironCalcRef: RefObject<IronCalcHandle | null>;
  model: Model;
  isReadOnly: boolean;
  isOpen: boolean;
  onOpen: () => void;
  onClose: () => void;
  onHover: () => void;
}) {
  const [anchorPosition, setAnchorPosition] = useState({ x: 0, y: 0 });
  const triggerRef = useRef<HTMLButtonElement>(null);
  const { t } = useTranslation();
  const { model, isReadOnly } = props;

  const captureAnchor = () => {
    const rect = triggerRef.current?.getBoundingClientRect();
    if (rect) {
      setAnchorPosition({ x: rect.left, y: rect.bottom + 4 });
    }
  };

  const actions = () => props.ironCalcRef.current?.workbook;

  return (
    <div>
      <Button
        ref={triggerRef}
        variant="ghost"
        id="insert-button"
        aria-haspopup="menu"
        aria-expanded={props.isOpen ? "true" : "false"}
        onClick={() => {
          captureAnchor();
          props.onOpen();
        }}
        onMouseEnter={() => {
          captureAnchor();
          props.onHover();
        }}
      >
        {t("file_bar.insert_menu.button")}
      </Button>

      <Menu
        open={props.isOpen}
        onClose={props.onClose}
        anchorPosition={anchorPosition}
      >
        <MenuItemWithSubmenu
          icon={<Rows3 />}
          submenu={
            <>
              <MenuItem
                disabled={isReadOnly}
                onClick={() => actions()?.insertRowsAbove()}
              >
                {t("file_bar.insert_menu.row_above")}
              </MenuItem>
              <MenuItem
                disabled={isReadOnly}
                onClick={() => actions()?.insertRowsBelow()}
              >
                {t("file_bar.insert_menu.row_below")}
              </MenuItem>
            </>
          }
        >
          {t("file_bar.insert_menu.rows")}
        </MenuItemWithSubmenu>
        <MenuItemWithSubmenu
          icon={<Columns3 />}
          submenu={
            <>
              <MenuItem
                disabled={isReadOnly}
                onClick={() => actions()?.insertColumnsLeft()}
              >
                {t("file_bar.insert_menu.column_left")}
              </MenuItem>
              <MenuItem
                disabled={isReadOnly}
                onClick={() => actions()?.insertColumnsRight()}
              >
                {t("file_bar.insert_menu.column_right")}
              </MenuItem>
            </>
          }
        >
          {t("file_bar.insert_menu.columns")}
        </MenuItemWithSubmenu>
        <MenuDivider />
        <MenuItem
          icon={<Sheet />}
          disabled={isReadOnly}
          onClick={() => {
            model.newSheet();
            actions()?.refresh();
          }}
        >
          {t("file_bar.insert_menu.sheet")}
        </MenuItem>
      </Menu>
    </div>
  );
}
