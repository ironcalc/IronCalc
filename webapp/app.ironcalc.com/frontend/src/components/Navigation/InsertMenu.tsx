import {
  Button,
  type IronCalcHandle,
  Menu,
  MenuDivider,
  MenuItem,
  type Model,
} from "@ironcalc/workbook";
import {
  BetweenHorizontalEnd,
  BetweenHorizontalStart,
  BetweenVerticalEnd,
  BetweenVerticalStart,
} from "lucide-react";
import { type RefObject, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

export function InsertMenu(props: {
  // The widget's handle: the menu runs its actions through it
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

  // As many rows (columns) are inserted as the selection spans, so the labels
  // carry the count
  const {
    range: [rowStart, columnStart, rowEnd, columnEnd],
  } = model.getSelectedView();
  const rowCount = Math.abs(rowEnd - rowStart) + 1;
  const columnCount = Math.abs(columnEnd - columnStart) + 1;

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
        <MenuItem
          icon={<BetweenHorizontalStart />}
          disabled={isReadOnly}
          onClick={() => actions()?.insertRowsAbove()}
        >
          {t("file_bar.insert_menu.rows_above", { count: rowCount })}
        </MenuItem>
        <MenuItem
          icon={<BetweenHorizontalEnd />}
          disabled={isReadOnly}
          onClick={() => actions()?.insertRowsBelow()}
        >
          {t("file_bar.insert_menu.rows_below", { count: rowCount })}
        </MenuItem>
        <MenuDivider />
        <MenuItem
          icon={<BetweenVerticalStart />}
          disabled={isReadOnly}
          onClick={() => actions()?.insertColumnsLeft()}
        >
          {t("file_bar.insert_menu.columns_left", { count: columnCount })}
        </MenuItem>
        <MenuItem
          icon={<BetweenVerticalEnd />}
          disabled={isReadOnly}
          onClick={() => actions()?.insertColumnsRight()}
        >
          {t("file_bar.insert_menu.columns_right", { count: columnCount })}
        </MenuItem>
      </Menu>
    </div>
  );
}
