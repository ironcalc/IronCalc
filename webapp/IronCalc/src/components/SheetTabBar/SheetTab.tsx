import {
  ChevronDown,
  EyeOff,
  PaintBucket,
  TextCursorInput,
  Trash2,
} from "lucide-react";
import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import ColorPicker from "../ColorPicker/ColorPicker";
import { isInReferenceMode } from "../Editor/util";
import type { WorkbookState } from "../workbookState";
import SheetDeleteDialog from "./SheetDeleteDialog";
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

  const [isMenuOpen, setMenuOpen] = useState(false);
  const [menuStyle, setMenuStyle] = useState<{
    left?: number;
    bottom?: number;
  }>({});
  const [colorPickerOpen, setColorPickerOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);

  const [isEditing, setIsEditing] = useState(false);
  const [editingName, setEditingName] = useState(name);
  const [inputWidth, setInputWidth] = useState<number>(0);

  const tabRef = useRef<HTMLDivElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const measureRef = useRef<HTMLSpanElement>(null);

  const focusMenuItem = useCallback((index: number) => {
    const items =
      menuRef.current?.querySelectorAll<HTMLButtonElement>(":scope > button");

    if (!items || items.length === 0) {
      return;
    }

    const safeIndex = Math.max(0, Math.min(index, items.length - 1));
    items[safeIndex]?.focus();
  }, []);

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

  // We want to change the layout only when editingName changes, but the layout is controlled by the hidden measure element.
  // biome-ignore lint/correctness/useExhaustiveDependencies: false
  useLayoutEffect(() => {
    if (isEditing && measureRef.current) {
      const width = measureRef.current.offsetWidth;
      setInputWidth(Math.max(width + 8, 6));
    }
  }, [editingName, isEditing]);

  useLayoutEffect(() => {
    if (!isMenuOpen || !tabRef.current) {
      return;
    }

    const updateMenuPosition = () => {
      const rect = tabRef.current?.getBoundingClientRect();

      if (!rect) {
        return;
      }

      setMenuStyle({
        // I try to align it with the left side of the chevron down
        left: rect.right - 24,
        bottom: window.innerHeight - rect.top,
      });
    };

    updateMenuPosition();

    requestAnimationFrame(() => {
      focusMenuItem(0);
    });

    window.addEventListener("resize", updateMenuPosition);
    window.addEventListener("scroll", updateMenuPosition, true);

    return () => {
      window.removeEventListener("resize", updateMenuPosition);
      window.removeEventListener("scroll", updateMenuPosition, true);
    };
  }, [isMenuOpen, focusMenuItem]);

  useEffect(() => {
    if (!isMenuOpen) {
      return;
    }

    const onDocumentPointerDown = (event: PointerEvent) => {
      const target = event.target as Node | null;

      if (
        tabRef.current?.contains(target) ||
        menuRef.current?.contains(target)
      ) {
        return;
      }

      setMenuOpen(false);
    };

    document.addEventListener("pointerdown", onDocumentPointerDown, true);

    return () => {
      document.removeEventListener("pointerdown", onDocumentPointerDown, true);
    };
  }, [isMenuOpen]);

  const handleOpenMenu = (event: React.MouseEvent) => {
    event.stopPropagation();
    event.preventDefault();
    setMenuOpen(true);
  };

  const handleCloseMenu = () => {
    setMenuOpen(false);
  };

  const handleContextMenu = (event: React.MouseEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.stopPropagation();
    onSelected();
    setMenuOpen(true);
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
      {/** biome-ignore lint/a11y/noStaticElementInteractions: FIXME */}
      {/** biome-ignore lint/a11y/useKeyWithClickEvents: FIXME */}
      <div
        className={`ic-sheet-tab${selected ? " ic-sheet-tab--selected" : ""}`}
        style={{ borderBottomColor: color }}
        onClick={(event) => {
          if (!isEditing) {
            onSelected();
          }
          event.stopPropagation();
          event.preventDefault();
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
              className={`ic-sheet-tab-menu-button${isMenuOpen ? " ic-sheet-tab-menu-button--active" : ""}`}
              onClick={handleOpenMenu}
              type="button"
              aria-label={t("sheet_tab.open_menu")}
            >
              <ChevronDown />
            </button>
          </>
        )}
      </div>

      {isMenuOpen ? (
        <div
          className="ic-sheet-tab-menu"
          ref={menuRef}
          style={menuStyle}
          role="menu"
          aria-label={t("sheet_tab.open_menu")}
          onKeyDown={(event) => {
            const items =
              menuRef.current?.querySelectorAll<HTMLButtonElement>(
                ":scope > button",
              );

            if (!items || items.length === 0) {
              return;
            }

            const itemsArray = Array.from(items);
            const currentIndex = itemsArray.indexOf(
              document.activeElement as HTMLButtonElement,
            );

            switch (event.key) {
              case "Escape":
                event.preventDefault();
                handleCloseMenu();
                break;
              case "ArrowDown":
                event.preventDefault();
                focusMenuItem((currentIndex + 1 + items.length) % items.length);
                break;
              case "ArrowUp":
                event.preventDefault();
                focusMenuItem((currentIndex - 1 + items.length) % items.length);
                break;
              case "Home":
                event.preventDefault();
                focusMenuItem(0);
                break;
              case "End":
                event.preventDefault();
                focusMenuItem(items.length - 1);
                break;
              case "Tab":
                handleCloseMenu();
                break;
              default:
                break;
            }
          }}
        >
          <button
            className="ic-sheet-tab-menu-item"
            onClick={() => {
              handleStartEditing();
              handleCloseMenu();
            }}
            type="button"
          >
            <TextCursorInput />
            {t("sheet_tab.rename")}
          </button>

          <button
            className="ic-sheet-tab-menu-item"
            onClick={() => {
              setColorPickerOpen(true);
              handleCloseMenu();
            }}
            type="button"
          >
            <PaintBucket />
            {t("sheet_tab.change_color")}
          </button>

          <button
            className="ic-sheet-tab-menu-item"
            disabled={!props.canDelete}
            onClick={() => {
              props.onHideSheet();
              handleCloseMenu();
            }}
            type="button"
          >
            <EyeOff />
            {t("sheet_tab.hide_sheet")}
          </button>

          <div className="ic-sheet-tab-menu-divider" />

          <button
            className="ic-sheet-tab-menu-item ic-sheet-tab-menu-item--delete"
            disabled={!props.canDelete}
            onClick={() => {
              setDeleteDialogOpen(true);
              handleCloseMenu();
            }}
            type="button"
          >
            <Trash2 />
            {t("sheet_tab.delete")}
          </button>
        </div>
      ) : null}

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
        anchorOrigin={{ vertical: "bottom", horizontal: "right" }}
        transformOrigin={{ vertical: "bottom", horizontal: "left" }}
      />

      <SheetDeleteDialog
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
