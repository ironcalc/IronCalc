import { ChevronDown } from "lucide-react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import ColorPicker from "../ColorPicker/ColorPicker";
import { isInReferenceMode } from "../Editor/util";
import { Menu } from "../Menu/Menu";
import type { WorkbookState } from "../workbookState";
import SheetDeleteModal from "./SheetDeleteModal";
import { SheetTabMenu } from "./SheetTabMenu";
import "./sheet-tab.css";

interface SheetTabProps {
  name: string;
  color: string;
  selected: boolean;
  onSelected: () => void;
  onColorChanged: (hex: string) => void;
  onRenamed: (name: string) => void;
  canDelete: boolean;
  onDeleted: () => void;
  onHideSheet: () => void;
  workbookState: WorkbookState;
}

function SheetTab(props: SheetTabProps) {
  const { name, color, selected, workbookState, onSelected } = props;
  const { t } = useTranslation();

  const [menuOpen, setMenuOpen] = useState(false);
  const [menuAnchorPosition, setMenuAnchorPosition] = useState({ x: 0, y: 0 });
  const [colorPickerOpen, setColorPickerOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);

  const [isEditing, setIsEditing] = useState(false);
  const [editingName, setEditingName] = useState(name);
  const [inputWidth, setInputWidth] = useState<number>(0);

  const tabRef = useRef<HTMLDivElement>(null);
  const menuButtonRef = useRef<HTMLButtonElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const measureRef = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditing]);

  useEffect(() => {
    if (!isEditing) {
      setEditingName(name);
    }
  }, [name, isEditing]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: false
  useLayoutEffect(() => {
    if (isEditing && measureRef.current) {
      const width = measureRef.current.offsetWidth;
      setInputWidth(Math.max(width + 8, 6));
    }
  }, [editingName, isEditing]);

  function getMenuAnchorPosition() {
    const rect = tabRef.current?.getBoundingClientRect();
    if (!rect) return { x: 0, y: 0 };
    return { x: rect.right - 24, y: rect.top };
  }

  const handleOpenMenu = (event: React.MouseEvent) => {
    event.stopPropagation();
    event.preventDefault();
    if (menuOpen) {
      setMenuOpen(false);
      requestAnimationFrame(() => menuButtonRef.current?.focus());
    } else {
      onSelected();
      setMenuAnchorPosition(getMenuAnchorPosition());
      setMenuOpen(true);
    }
  };

  const handleContextMenu = (event: React.MouseEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.stopPropagation();
    onSelected();
    setMenuAnchorPosition(getMenuAnchorPosition());
    setMenuOpen(true);
  };

  const handleCloseMenu = () => {
    setMenuOpen(false);
    requestAnimationFrame(() => menuButtonRef.current?.focus());
  };

  const handleStartEditing = () => {
    setEditingName(name);
    setInputWidth(Math.max(name.length * 7 + 8, 6));
    setIsEditing(true);
  };

  const handleSave = () => {
    if (editingName.trim() !== "") {
      props.onRenamed(editingName.trim());
      setIsEditing(false);
    } else {
      setEditingName(name);
      setIsEditing(false);
    }
  };

  const handleCancel = () => {
    setEditingName(name);
    setIsEditing(false);
  };

  return (
    <>
      <div
        className={`ic-sheet-tab${selected ? " ic-sheet-tab--selected" : ""}`}
        style={{ borderBottomColor: color }}
        role="tab"
        tabIndex={selected ? 0 : -1}
        aria-selected={selected}
        onClick={(event) => {
          if (!isEditing) {
            onSelected();
          }
          event.stopPropagation();
          event.preventDefault();
        }}
        onKeyDown={(_) => {
          // not handling enter/space to open menu or start editing here,
        }}
        onDoubleClick={(event) => {
          event.stopPropagation();
          event.preventDefault();
          handleStartEditing();
        }}
        onContextMenu={handleContextMenu}
        onPointerDown={(event) => {
          const cell = workbookState.getEditingCell();
          if (cell && isInReferenceMode(cell.text, cell.cursorStart)) {
            event.stopPropagation();
            event.preventDefault();
          }
        }}
        ref={tabRef}
      >
        {isEditing ? (
          <>
            <span className="ic-sheet-tab-hidden-measure" ref={measureRef}>
              {editingName || " "}
            </span>
            <input
              ref={inputRef}
              value={editingName}
              onChange={(e) => setEditingName(e.target.value)}
              style={{ width: `${inputWidth}px` }}
              className="ic-sheet-tab-input"
              aria-label={t("sheet_tab.rename")}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  handleSave();
                } else if (e.key === "Escape") {
                  e.preventDefault();
                  handleCancel();
                }
                e.stopPropagation();
              }}
              onBlur={() => {
                handleSave();
              }}
              onClick={(e) => e.stopPropagation()}
              spellCheck={false}
            />
            {/** biome-ignore lint/a11y/noAriaHiddenOnFocusable: FIXME */}
            <button
              className="ic-sheet-tab-menu-button"
              disabled
              type="button"
              aria-hidden="true"
            >
              <ChevronDown />
            </button>
          </>
        ) : (
          <>
            <div className="ic-sheet-tab-name">{name}</div>
            <button
              ref={menuButtonRef}
              className={`ic-sheet-tab-menu-button${menuOpen ? " ic-sheet-tab-menu-button--active" : ""}`}
              onClick={handleOpenMenu}
              type="button"
              aria-label={t("sheet_tab.open_menu")}
              aria-haspopup="menu"
              aria-expanded={menuOpen}
            >
              <ChevronDown />
            </button>
          </>
        )}
      </div>

      <Menu
        open={menuOpen}
        onClose={handleCloseMenu}
        anchorPosition={menuAnchorPosition}
      >
        <SheetTabMenu
          canDelete={props.canDelete}
          onStartEditing={handleStartEditing}
          onOpenColorPicker={() => setColorPickerOpen(true)}
          onHideSheet={props.onHideSheet}
          onDeleteSheet={() => setDeleteDialogOpen(true)}
        />
      </Menu>

      <ColorPicker
        color={color}
        defaultColor="#FFFFFF"
        title={t("color_picker.no_fill")}
        onChange={(nextColor): void => {
          props.onColorChanged(nextColor);
          setColorPickerOpen(false);
        }}
        onClose={() => {
          setColorPickerOpen(false);
        }}
        anchorEl={tabRef}
        open={colorPickerOpen}
        placement="top"
      />

      <SheetDeleteModal
        open={deleteDialogOpen}
        onClose={() => setDeleteDialogOpen(false)}
        onDelete={() => {
          props.onDeleted();
          setDeleteDialogOpen(false);
        }}
        sheetName={name}
      />
    </>
  );
}

export default SheetTab;
