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
import { Checkbox } from "../Checkbox/Checkbox";
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

function bytesEqual(a: Uint8Array, b: Uint8Array): boolean {
  return a.length === b.length && a.every((value, i) => value === b[i]);
}

interface Peer {
  name: string;
  model: Model;
  // Stable across renders so editing state survives re-renders of the story
  workbookState: WorkbookState;
  actions: React.RefObject<WorkbookActions | null>;
}

const MIN_DELAY_SECONDS = 3;
const MAX_DELAY_SECONDS = 20;

// Two workbooks of the same document, kept in sync the way two remote peers
// would be: the diffs each model queues (`flushSendQueue`) are applied to the
// other model (`applyExternalDiffs`) and the other widget is repainted through
// `WorkbookActions.refresh`. The widget does not report changes to the host,
// so the "network" is a poll of the send queues.
//
// Peer B can hold incoming diffs for a few seconds. That leaves time to start
// an edit in Peer B and see a change from Peer A land while the edit is in
// progress, without the edit being lost.
function SyncedWorkbooks({ canEdit }: { canEdit: boolean }) {
  const [peers, setPeers] = useState<[Peer, Peer] | null>(null);
  const [received, setReceived] = useState<[number, number]>([0, 0]);
  const [delayIncoming, setDelayIncoming] = useState(false);
  const [delaySeconds, setDelaySeconds] = useState(MIN_DELAY_SECONDS);
  // The sync timer reads this ref, so changing the delay does not restart it
  const delayRef = useRef(0);
  delayRef.current = delayIncoming ? delaySeconds * 1000 : 0;
  const actionsA = useRef<WorkbookActions>(null);
  const actionsB = useRef<WorkbookActions>(null);

  useEffect(() => {
    async function start() {
      await init();
      const modelA = await loadModel();
      const modelB = Model.fromBytes(modelA.toBytes(), "en");
      setPeers([
        {
          name: "Peer A",
          model: modelA,
          workbookState: new WorkbookState(),
          actions: actionsA,
        },
        {
          name: "Peer B",
          model: modelB,
          workbookState: new WorkbookState(),
          actions: actionsB,
        },
      ]);
    }
    start();
  }, []);

  useEffect(() => {
    if (!peers) {
      return;
    }
    // What an empty queue encodes to, so we only ship actual changes
    const emptyQueue = peers[0].model.flushSendQueue();
    peers[1].model.flushSendQueue();

    const send = (from: Peer, to: Peer, toIndex: 0 | 1) => {
      const diffs = from.model.flushSendQueue();
      if (bytesEqual(diffs, emptyQueue)) {
        return;
      }
      const apply = () => {
        to.model.applyExternalDiffs(diffs);
        to.actions.current?.refresh();
        setReceived((counts) => {
          const next: [number, number] = [...counts];
          next[toIndex] += 1;
          return next;
        });
      };
      if (toIndex === 1 && delayRef.current > 0) {
        setTimeout(apply, delayRef.current);
      } else {
        apply();
      }
    };

    const timer = setInterval(() => {
      send(peers[0], peers[1], 1);
      send(peers[1], peers[0], 0);
    }, 100);
    return () => clearInterval(timer);
  }, [peers]);

  if (!peers) {
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
        gap: 8,
        fontFamily: "var(--typography-font-family)",
        fontSize: 12,
        color: "var(--palette-grey-600)",
      }}
    >
      {peers.map((peer, index) => (
        <div
          key={peer.name}
          style={{
            flex: 1,
            minWidth: 0,
            display: "flex",
            flexDirection: "column",
            borderLeft: index ? "1px solid var(--palette-grey-300)" : "none",
          }}
        >
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 12,
              padding: "4px 12px",
              minHeight: 36,
            }}
          >
            <span>
              <strong>{peer.name}</strong> · received {received[index]} diff
              batches
            </span>
            {index === 1 && (
              <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                <Checkbox
                  checked={delayIncoming}
                  onChange={setDelayIncoming}
                  label="Delay:"
                />
                <span>{MIN_DELAY_SECONDS} s</span>
                <input
                  type="range"
                  min={MIN_DELAY_SECONDS}
                  max={MAX_DELAY_SECONDS}
                  value={delaySeconds}
                  disabled={!delayIncoming}
                  onChange={(event) =>
                    setDelaySeconds(Number(event.target.value))
                  }
                />
                <span>{MAX_DELAY_SECONDS} s</span>
                <strong style={{ minWidth: "3ch" }}>{delaySeconds} s</strong>
              </div>
            )}
          </div>
          <div className="ic-widget" style={{ flex: 1, position: "relative" }}>
            <Workbook
              ref={peer.actions}
              model={peer.model}
              workbookState={peer.workbookState}
              canEdit={canEdit}
            />
          </div>
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

/**
 * Two workbooks kept in sync as if they were two peers: every change made in
 * one is sent as diffs to the other and the other is repainted through the
 * imperative handle, without disturbing what the user is doing there.
 */
export const SyncedPeers: Story = {
  render: (args) => <SyncedWorkbooks {...args} />,
};
