import type { Meta, StoryObj } from "@storybook/react";
import {
  Bold,
  ClipboardPaste,
  Copy,
  Eraser,
  Italic,
  Paintbrush,
  Redo2,
  Scissors,
  Strikethrough,
  Underline,
  Undo2,
} from "lucide-react";
import { type ReactNode, useEffect, useRef, useState } from "react";
import { init, Model } from "../../index";
import { Button } from "../Button/Button";
import { WorkbookState } from "../workbookState";
import Workbook from "./Workbook";
import type { WorkbookActions } from "./workbookActions";

// Optionally load the example workbook in webapp/IronCalc/tests/example.ic if
// present. The file is gitignored, so this glob resolves to an empty object
// when the file is missing and we fall back to a new empty workbook.
const exampleWorkbooks = import.meta.glob("../../../tests/example.ic", {
  query: "?url",
  import: "default",
  eager: true,
}) as Record<string, string>;

async function loadModel(): Promise<Model> {
  const exampleUrl = Object.values(exampleWorkbooks)[0];
  if (exampleUrl) {
    try {
      const response = await fetch(exampleUrl);
      if (response.ok) {
        const bytes = new Uint8Array(await response.arrayBuffer());
        return Model.fromBytes(bytes, "en");
      }
    } catch {
      // Fall through to a new empty workbook on any fetch/parse error.
    }
  }
  return new Model("Workbook1", "en", "UTC", "en");
}

function useModel(): Model | null {
  const [model, setModel] = useState<Model | null>(null);

  useEffect(() => {
    async function start() {
      await init();
      setModel(await loadModel());
    }
    start();
  }, []);

  return model;
}

function WorkbookWithInit({ canEdit }: { canEdit: boolean }) {
  const model = useModel();

  if (!model) {
    return <div>Loading...</div>;
  }
  return (
    <div
      className="ic-widget"
      style={{
        position: "absolute",
        top: 0,
        bottom: 0,
        left: 0,
        right: 0,
      }}
    >
      <Workbook
        model={model}
        workbookState={new WorkbookState()}
        canEdit={canEdit}
      />
    </div>
  );
}

// A bar of buttons that drives the workbook through its imperative handle
// (`WorkbookActions`), the same way a host application would. Each button
// acts on the current selection, so select some cells first.
function ImperativeHandleBar({
  actions,
}: {
  actions: React.RefObject<WorkbookActions | null>;
}) {
  const run = (action: (actions: WorkbookActions) => void) => () => {
    if (actions.current) {
      action(actions.current);
    }
  };

  const groups: {
    name: string;
    buttons: [string, ReactNode, (a: WorkbookActions) => void][];
  }[] = [
    {
      name: "History",
      buttons: [
        ["Undo", <Undo2 key="undo" />, (a) => a.undo()],
        ["Redo", <Redo2 key="redo" />, (a) => a.redo()],
      ],
    },
    {
      name: "Clipboard",
      buttons: [
        ["Cut", <Scissors key="cut" />, (a) => a.cut()],
        ["Copy", <Copy key="copy" />, (a) => a.copy()],
        ["Paste", <ClipboardPaste key="paste" />, (a) => a.paste()],
      ],
    },
    {
      name: "Font",
      buttons: [
        ["Bold", <Bold key="bold" />, (a) => a.toggleBold()],
        ["Italic", <Italic key="italic" />, (a) => a.toggleItalic()],
        [
          "Underline",
          <Underline key="underline" />,
          (a) => a.toggleUnderline(),
        ],
        [
          "Strikethrough",
          <Strikethrough key="strike" />,
          (a) => a.toggleStrikethrough(),
        ],
        [
          "Clear formatting",
          <Eraser key="clear" />,
          (a) => a.clearFormatting(),
        ],
      ],
    },
    {
      name: "Insert",
      buttons: [
        ["Rows above", undefined, (a) => a.insertRowsAbove()],
        ["Rows below", undefined, (a) => a.insertRowsBelow()],
        ["Columns left", undefined, (a) => a.insertColumnsLeft()],
        ["Columns right", undefined, (a) => a.insertColumnsRight()],
      ],
    },
    {
      name: "Dialogs",
      buttons: [
        [
          "Conditional formatting",
          <Paintbrush key="cf" />,
          (a) => a.openConditionalFormatting(),
        ],
      ],
    },
  ];

  return (
    <div
      style={{
        display: "flex",
        flexWrap: "wrap",
        alignItems: "center",
        gap: 16,
        padding: "8px 12px",
        borderBottom: "1px solid var(--palette-grey-300)",
        fontFamily: "var(--typography-font-family)",
        fontSize: 12,
        color: "var(--palette-grey-600)",
      }}
    >
      {groups.map((group) => (
        <div
          key={group.name}
          style={{ display: "flex", alignItems: "center", gap: 4 }}
        >
          <span style={{ marginRight: 4 }}>{group.name}</span>
          {group.buttons.map(([label, icon, action]) => (
            <Button
              key={label}
              variant="outline"
              size="sm"
              startIcon={icon}
              onClick={run(action)}
            >
              {label}
            </Button>
          ))}
        </div>
      ))}
    </div>
  );
}

function WorkbookWithImperativeHandle({ canEdit }: { canEdit: boolean }) {
  const model = useModel();
  const actionsRef = useRef<WorkbookActions>(null);

  if (!model) {
    return <div>Loading...</div>;
  }
  return (
    <div
      style={{
        position: "absolute",
        top: 0,
        bottom: 0,
        left: 0,
        right: 0,
        display: "flex",
        flexDirection: "column",
      }}
    >
      <ImperativeHandleBar actions={actionsRef} />
      <div className="ic-widget" style={{ flex: 1, position: "relative" }}>
        <Workbook
          ref={actionsRef}
          model={model}
          workbookState={new WorkbookState()}
          canEdit={canEdit}
        />
      </div>
    </div>
  );
}

const meta = {
  title: "Components/Workbook",
  component: WorkbookWithInit,
  parameters: {
    layout: "fullscreen",
  },
  argTypes: {
    canEdit: { control: "boolean" },
  },
  args: {
    canEdit: true,
  },
} satisfies Meta<typeof WorkbookWithInit>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {};

/**
 * The workbook driven from the outside through its imperative handle, the
 * way a host application (for instance an "Edit" menu) would use it.
 */
export const ImperativeHandle: Story = {
  render: (args) => <WorkbookWithImperativeHandle {...args} />,
};
