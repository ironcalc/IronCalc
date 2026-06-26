import type { Color, Model } from "@ironcalc/wasm";
import { Menu as MenuIcon, Plus, Settings } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "../Button/Button";
import { IconButton } from "../Button/IconButton";
import { Menu } from "../Menu/Menu";
import { getLocaleDisplayName } from "../RightDrawer/RegionalSettings/RegionalSettings";
import { Tooltip } from "../Tooltip/Tooltip";
import type { WorkbookState } from "../workbookState";
import SheetListMenu from "./SheetListMenu";
import SheetTab from "./SheetTab";
import type { SheetOptions } from "./types";
import "./sheet-tab-bar.css";

export interface SheetTabBarProps {
  sheets: SheetOptions[];
  selectedIndex: number;
  workbookState: WorkbookState;
  onSheetSelected: (index: number) => void;
  onAddBlankSheet: () => void;
  onSheetColorChanged: (color: Color) => void;
  onSheetRenamed: (name: string) => void;
  onSheetDeleted: () => void;
  onSheetDuplicated: () => void;
  onHideSheet: () => void;
  model: Model;
  onOpenRegionalSettings: () => void;
}

function SheetTabBar(props: SheetTabBarProps) {
  const { t } = useTranslation();
  const { workbookState, onSheetSelected, sheets, selectedIndex } = props;
  const nonHiddenSheets = sheets
    .map((s, index) => {
      return {
        state: s.state,
        index,
        name: s.name,
        color: s.color,
        sheetId: s.sheetId,
      };
    })
    .filter((s) => s.state === "visible");

  return (
    <div className="ic-sheet-tab-bar-container">
      <div className="ic-sheet-tab-bar-left-buttons-container">
        <Tooltip title={t("navigation.add_sheet")}>
          <IconButton
            aria-label={t("navigation.add_sheet")}
            icon={<Plus />}
            onClick={props.onAddBlankSheet}
          />
        </Tooltip>
        <Tooltip title={t("navigation.sheet_list")}>
          <Menu
            trigger={
              <IconButton
                aria-label={t("navigation.sheet_list")}
                icon={<MenuIcon />}
              />
            }
          >
            <SheetListMenu
              sheetOptionsList={sheets}
              onSheetSelected={onSheetSelected}
              selectedIndex={selectedIndex}
            />
          </Menu>
        </Tooltip>
      </div>
      <div className="ic-sheet-tab-bar-vertical-divider" />
      <div className="ic-sheet-tab-bar-sheets">
        <div className="ic-sheet-tab-bar-sheet-inner">
          {nonHiddenSheets.map((tab) => (
            <SheetTab
              key={tab.sheetId}
              name={tab.name}
              color={tab.color}
              selected={tab.index === selectedIndex}
              onSelected={() => onSheetSelected(tab.index)}
              onColorChanged={(color) => {
                props.onSheetColorChanged(color);
              }}
              onRenamed={(name: string): void => {
                props.onSheetRenamed(name);
              }}
              canDelete={nonHiddenSheets.length > 1}
              onDeleted={(): void => {
                props.onSheetDeleted();
              }}
              onDuplicateSheet={(): void => {
                props.onSheetDuplicated();
              }}
              onHideSheet={props.onHideSheet}
              workbookState={workbookState}
              currentTheme={props.model.getTheme()}
            />
          ))}
        </div>
      </div>
      <div className="ic-sheet-tab-bar-right-container">
        <Tooltip title={t("regional_settings.open_regional_settings")}>
          <Button
            className="ic-sheet-tab-bar-regional-settings-button"
            variant="ghost"
            size="sm"
            onClick={() => {
              props.onOpenRegionalSettings();
            }}
          >
            {getLocaleDisplayName(props.model.getLocale())}
            <div className="ic-sheet-tab-bar-text-divider" />
            {t(
              `regional_settings.language.display_language.${props.model.getLanguage()}`,
            )}
          </Button>
        </Tooltip>
        <Tooltip title={t("regional_settings.open_regional_settings")}>
          <IconButton
            className="ic-sheet-tab-bar-regional-settings-icon-button"
            aria-label={t("regional_settings.open_regional_settings")}
            icon={<Settings />}
            onClick={props.onOpenRegionalSettings}
          />
        </Tooltip>
      </div>
    </div>
  );
}

export default SheetTabBar;
