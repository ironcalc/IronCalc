import type { Model } from "@ironcalc/wasm";
import { ChevronDown } from "lucide-react";
import { useState } from "react";
import { Fx } from "../../icons";
import { Button } from "../Button/Button";
import Editor from "../Editor/Editor";
import {
  COLUMN_WIDTH_SCALE,
  ROW_HEIGH_SCALE,
} from "../WorksheetCanvas/constants";
import type { WorkbookState } from "../workbookState";
import FormulaBarMenu from "./FormulaBarMenu";
import "./formula-bar.css";

type FormulaBarProps = {
  cellAddress: string;
  formulaValue: string;
  model: Model;
  workbookState: WorkbookState;
  onChange: () => void;
  onTextUpdated: () => void;
  openDrawer: () => void;
  canEdit: boolean;
};

function FormulaBar(properties: FormulaBarProps) {
  const {
    cellAddress,
    formulaValue,
    model,
    onChange,
    onTextUpdated,
    workbookState,
  } = properties;
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  const handleMenuOpenChange = (isOpen: boolean): void => {
    setIsMenuOpen(isOpen);
  };

  return (
    <div className="ic-formula-bar-root">
      <div
        className={`${isMenuOpen ? "ic-formula-bar-address ic-formula-bar-address--active" : "ic-formula-bar-address"}`}
      >
        <FormulaBarMenu
          onMenuOpenChange={handleMenuOpenChange}
          openDrawer={properties.openDrawer}
          canEdit={properties.canEdit}
          model={model}
          onUpdate={onChange}
        >
          <Button
            variant="ghost"
            size="sm"
            pressed={isMenuOpen}
            endIcon={<ChevronDown />}
          >
            {cellAddress}
          </Button>
        </FormulaBarMenu>
      </div>
      <div className="ic-formula-bar-divider" />
      <div className="ic-formula-bar-formula-container">
        <div className="ic-formula-bar-button">
          <Fx />
        </div>
        {/** biome-ignore lint/a11y/noStaticElementInteractions: FIXME */}
        {/** biome-ignore lint/a11y/useKeyWithClickEvents: FIXME */}
        <div
          className="ic-formula-bar-editor-wrapper"
          onClick={(event) => {
            const [sheet, row, column] = model.getSelectedCell();
            const editorWidth =
              model.getColumnWidth(sheet, column) * COLUMN_WIDTH_SCALE;
            const editorHeight =
              model.getRowHeight(sheet, row) * ROW_HEIGH_SCALE;
            workbookState.setEditingCell({
              sheet,
              row,
              column,
              text: formulaValue,
              referencedRange: null,
              cursorStart: formulaValue.length,
              cursorEnd: formulaValue.length,
              focus: "formula-bar",
              activeRanges: [],
              mode: "edit",
              editorWidth,
              editorHeight,
            });
            event.stopPropagation();
            event.preventDefault();
          }}
        >
          <Editor
            originalText={formulaValue}
            model={model}
            workbookState={workbookState}
            onEditEnd={() => {
              onChange();
            }}
            onTextUpdated={onTextUpdated}
            type="formula-bar"
          />
        </div>
      </div>
    </div>
  );
}

export default FormulaBar;
