import {
  Button,
  type IronCalcHandle,
  Menu,
  MenuDivider,
  MenuItem,
  MenuItemWithSubmenu,
  type Model,
} from "@ironcalc/workbook";
import {
  EyeOff,
  Grid2x2Check,
  Grid2x2X,
  ImageDown,
  Snowflake,
  Tag,
} from "lucide-react";
import { type RefObject, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

export function ViewMenu(props: {
  ironCalcRef: RefObject<IronCalcHandle | null>;
  model: Model;
  isOpen: boolean;
  onOpen: () => void;
  onClose: () => void;
  onHover: () => void;
}) {
  const [anchorPosition, setAnchorPosition] = useState({ x: 0, y: 0 });
  const triggerRef = useRef<HTMLButtonElement>(null);
  const { t } = useTranslation();
  const { model } = props;

  const captureAnchor = () => {
    const rect = triggerRef.current?.getBoundingClientRect();
    if (rect) {
      setAnchorPosition({ x: rect.left, y: rect.bottom + 4 });
    }
  };

  const update = (change: () => void) => {
    change();
    props.ironCalcRef.current?.workbook?.refresh();
  };

  const sheet = model.getSelectedSheet();
  const showGridLines = model.getShowGridLines(sheet);
  const hiddenSheets = model
    .getWorksheetsProperties()
    .map(({ name, state }, index) => ({ name, state, index }))
    .filter(({ state }) => state === "hidden");

  return (
    <div>
      <Button
        ref={triggerRef}
        variant="ghost"
        id="view-button"
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
        {t("file_bar.view_menu.button")}
      </Button>

      <Menu
        open={props.isOpen}
        onClose={props.onClose}
        anchorPosition={anchorPosition}
      >
        <MenuItem
          icon={showGridLines ? <Grid2x2X /> : <Grid2x2Check />}
          onClick={() =>
            update(() => model.setShowGridLines(sheet, !showGridLines))
          }
        >
          {t(
            showGridLines
              ? "file_bar.view_menu.hide_gridlines"
              : "file_bar.view_menu.show_gridlines",
          )}
        </MenuItem>
        <MenuDivider />
        <MenuItemWithSubmenu
          icon={<Snowflake />}
          submenu={
            <>
              {[1, 2].map((count) => (
                <MenuItem
                  key={count}
                  onClick={() =>
                    update(() => model.setFrozenRowsCount(sheet, count))
                  }
                >
                  {t("file_bar.view_menu.freeze_rows", { count })}
                </MenuItem>
              ))}
              <MenuDivider />
              {[1, 2].map((count) => (
                <MenuItem
                  key={count}
                  onClick={() =>
                    update(() => model.setFrozenColumnsCount(sheet, count))
                  }
                >
                  {t("file_bar.view_menu.freeze_columns", { count })}
                </MenuItem>
              ))}
            </>
          }
        >
          {t("file_bar.view_menu.freeze")}
        </MenuItemWithSubmenu>
        {hiddenSheets.length === 0 ? (
          <MenuItem icon={<EyeOff />} disabled>
            {t("file_bar.view_menu.hidden_sheets")}
          </MenuItem>
        ) : (
          <MenuItemWithSubmenu
            icon={<EyeOff />}
            submenu={hiddenSheets.map(({ name, index }) => (
              <MenuItem
                key={index}
                onClick={() =>
                  update(() => {
                    model.unhideSheet(index);
                    model.setSelectedSheet(index);
                  })
                }
              >
                {name}
              </MenuItem>
            ))}
          >
            {t("file_bar.view_menu.hidden_sheets")}
          </MenuItemWithSubmenu>
        )}
        <MenuDivider />
        <MenuItem
          icon={<Tag />}
          onClick={() => props.ironCalcRef.current?.workbook?.openNamedRanges()}
        >
          {t("file_bar.view_menu.named_ranges")}
        </MenuItem>
        <MenuDivider />
        <MenuItem
          icon={<ImageDown />}
          onClick={() =>
            props.ironCalcRef.current?.workbook?.downloadSelectionAsPng()
          }
        >
          {t("file_bar.view_menu.download_png")}
        </MenuItem>
      </Menu>
    </div>
  );
}
