import type { Meta, StoryObj } from "@storybook/react";
import { useEffect, useState } from "react";
import { init, Model } from "../../index";
import {
  COLUMN_WIDTH_SCALE,
  ROW_HEIGH_SCALE,
} from "../WorksheetCanvas/constants";
import { WorkbookState } from "../workbookState";
import Editor from "./Editor";

// The smallest faithful harness for the formula editor: just the <Editor> and
// the wrapper whose onClick starts an edit (this is what FormulaBar does around
// the editor, minus the name box and the rest of the bar). Re-renders are driven
// by onChange/onTextUpdated bumping a counter, and `formulaValue` is recomputed
// each render, returning the live editing text while editing.
function EditorHarness() {
  const [model, setModel] = useState<Model | null>(null);
  const [workbookState] = useState(() => new WorkbookState());
  const [, setRedrawId] = useState(0);
  const redraw = () => setRedrawId((id) => id + 1);

  useEffect(() => {
    async function start() {
      await init();
      const m = new Model("Workbook1", "en", "UTC", "en");
      // Seed a cell with an editable formula so we can click in the middle.
      m.setUserInput(0, 1, 1, "=1+2+3");
      m.setSelectedCell(1, 1);
      setModel(m);
    }
    start();
  }, []);

  if (!model) {
    return <div>Loading…</div>;
  }

  const getFormulaValue = (): string => {
    const cell = workbookState.getEditingCell();
    if (cell) {
      return workbookState.getEditingText();
    }
    const { sheet, row, column } = model.getSelectedView();
    return model.getCellContent(sheet, row, column);
  };

  const formulaValue = getFormulaValue();

  return (
    <div
      style={{
        width: 400,
        height: 22,
        lineHeight: "22px",
        fontFamily: "monospace",
        border: "1px solid #ccc",
        background: "#fff",
      }}
    >
      {/* Mirrors FormulaBar's editor wrapper: clicking it starts an edit. */}
      <div
        style={{ position: "relative", width: "100%", height: "100%" }}
        onPointerDown={(event) => {
          const [sheet, row, column] = model.getSelectedCell();
          const editorWidth =
            model.getColumnWidth(sheet, column) * COLUMN_WIDTH_SCALE;
          const editorHeight = model.getRowHeight(sheet, row) * ROW_HEIGH_SCALE;
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
          // NOTE: no preventDefault() here — on pointerdown that would cancel
          // the textarea focus/caret placement, so the first click would set
          // the editing cell but never focus the editor.
        }}
      >
        <Editor
          originalText={formulaValue}
          model={model}
          workbookState={workbookState}
          onEditEnd={redraw}
          onTextUpdated={redraw}
          type="formula-bar"
          canEdit={true}
        />
      </div>
    </div>
  );
}

const meta = {
  title: "Components/Editor",
  component: EditorHarness,
  parameters: {
    layout: "centered",
  },
} satisfies Meta<typeof EditorHarness>;

export default meta;
type Story = StoryObj<typeof meta>;

export const FormulaBarEditor: Story = {};
