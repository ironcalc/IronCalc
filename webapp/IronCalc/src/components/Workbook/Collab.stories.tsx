import type { Meta, StoryObj } from "@storybook/react";
import { useEffect, useRef, useState } from "react";
import { IronCalc, init, Model } from "../../index";

const SYNC_INTERVAL_MS = 200;

interface PaneProps {
  label: string;
  model: Model;
  revision: number;
  offline: boolean;
  onOfflineChange: (offline: boolean) => void;
  /** Flex-grow share of the split layout. */
  grow: number;
}

function Pane({
  label,
  model,
  revision,
  offline,
  onOfflineChange,
  grow,
}: PaneProps) {
  const [container, setContainer] = useState<HTMLDivElement | null>(null);

  return (
    <div
      style={{
        flex: `${grow} 1 0px`,
        minWidth: 0,
        display: "flex",
        flexDirection: "column",
      }}
    >
      <div
        style={{ display: "flex", gap: 12, alignItems: "center", padding: 8 }}
      >
        <strong>{label}</strong>
        <label>
          <input
            type="checkbox"
            checked={offline}
            onChange={(event) => onOfflineChange(event.target.checked)}
          />
          offline
        </label>
      </div>
      {/* The widget needs its own root element, so mount it only once the ref is set.
          The canvas sizes itself to the window, so the pane must clip the overflow. */}
      <div
        ref={setContainer}
        style={{ flex: 1, position: "relative", overflow: "hidden" }}
      >
        {container ? (
          <IronCalc
            model={model}
            rootContainer={container}
            revision={revision}
          />
        ) : null}
      </div>
    </div>
  );
}

// The session-bound signatures exist only in the collab wasm build (`make collab`).
const CollabModel = Model as unknown as {
  new (
    name: string,
    locale: string,
    timezone: string,
    languageId: string,
    session: number,
  ): Model;
  fromBytes(bytes: Uint8Array, session: number): Model;
  fromXlsx(
    bytes: Uint8Array,
    name: string,
    locale: string,
    timezone: string,
    languageId: string,
    session: number,
  ): Model;
};

/** Minimal RFC 4180 parser: quoted fields, doubled quotes, CRLF or LF rows. */
function parseCsv(text: string): string[][] {
  const rows: string[][] = [];
  let row: string[] = [];
  let field = "";
  let quoted = false;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (quoted) {
      if (ch === '"') {
        if (text[i + 1] === '"') {
          field += '"';
          i++;
        } else {
          quoted = false;
        }
      } else {
        field += ch;
      }
    } else if (ch === '"') {
      quoted = true;
    } else if (ch === ",") {
      row.push(field);
      field = "";
    } else if (ch === "\n" || ch === "\r") {
      if (ch === "\r" && text[i + 1] === "\n") {
        i++;
      }
      row.push(field);
      rows.push(row);
      row = [];
      field = "";
    } else {
      field += ch;
    }
  }
  if (field !== "" || row.length > 0) {
    row.push(field);
    rows.push(row);
  }
  return rows;
}

function CollabDemo() {
  const [models, setModels] = useState<[Model, Model] | null>(null);
  const [revisions, setRevisions] = useState<[number, number]>([0, 0]);
  const [offline, setOffline] = useState<[boolean, boolean]>([false, false]);
  // Bumped when the models are replaced, so both widgets remount on the new pair.
  const [generation, setGeneration] = useState(0);
  const [status, setStatus] = useState("");
  // Share of the width held by the left pane, driven by the splitter bar.
  const [ratio, setRatio] = useState(0.5);
  const layoutRef = useRef<HTMLDivElement>(null);
  // The sync loop reads the latest offline flags without being restarted.
  const offlineRef = useRef(offline);
  offlineRef.current = offline;

  useEffect(() => {
    let cancelled = false;

    async function start() {
      await init();
      const a = new CollabModel("demo", "en", "UTC", "en", 1);
      const b = CollabModel.fromBytes(a.toBytes(), 2);
      if (!cancelled) {
        setModels([a, b]);
      }
    }

    start();
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!models) {
      return;
    }
    const [a, b] = models;
    const timer = setInterval(() => {
      // A checked box partitions that session in both directions. The pending
      // queue inside the model is the offline buffer, there is no JS queue.
      if (offlineRef.current[0] || offlineRef.current[1]) {
        return;
      }
      const transfer = (from: Model, to: Model, receiver: 0 | 1) => {
        const diffs = from.flushSendQueue();
        if (diffs.length === 0) {
          return;
        }
        const started = performance.now();
        to.applyExternalDiffs(diffs);
        const elapsed = Math.round(performance.now() - started);
        setStatus(
          `session ${receiver + 1} applied ${diffs.length} bytes in ${elapsed} ms`,
        );
        setRevisions((current) => {
          const next: [number, number] = [current[0], current[1]];
          next[receiver] += 1;
          return next;
        });
      };
      transfer(a, b, 1);
      transfer(b, a, 0);
    }, SYNC_INTERVAL_MS);
    return () => clearInterval(timer);
  }, [models]);

  if (!models) {
    return <div>Loading...</div>;
  }

  // An xlsx replaces both models: session 1 imports the file and session 2
  // joins from its snapshot, as at startup. (A fresh session 2 would not do: its
  // genesis sheet shares id 1 with the file's first sheet, and an AddSheet for a
  // known id is a no-op, so that sheet's content would be dropped on arrival.)
  // A csv is typed into the selected sheet of session 1 cell by cell, since csv
  // paste is not in the collab API.
  const loadFile = async (file: File) => {
    const bytes = new Uint8Array(await file.arrayBuffer());
    const started = performance.now();
    try {
      if (file.name.toLowerCase().endsWith(".csv")) {
        const [a] = models;
        const sheet = a.getSelectedSheet();
        let cells = 0;
        a.pauseEvaluation();
        parseCsv(new TextDecoder().decode(bytes)).forEach((row, r) => {
          row.forEach((value, c) => {
            if (value !== "") {
              a.setUserInput(sheet, r + 1, c + 1, value);
              cells++;
            }
          });
        });
        a.resumeEvaluation();
        // Resuming only lifts the pause: the batch is evaluated here, once.
        a.evaluate();
        setRevisions((current) => [current[0] + 1, current[1]]);
        setStatus(
          `${file.name}: ${cells} cells into session 1 in ${Math.round(performance.now() - started)} ms`,
        );
      } else {
        const a = CollabModel.fromXlsx(bytes, file.name, "en", "UTC", "en", 1);
        const b = CollabModel.fromBytes(a.toBytes(), 2);
        setModels([a, b]);
        setRevisions([0, 0]);
        setGeneration((g) => g + 1);
        setStatus(
          `${file.name}: imported into session 1 in ${Math.round(performance.now() - started)} ms, session 2 joined from its snapshot`,
        );
      }
    } catch (error) {
      setStatus(`${file.name}: ${error}`);
    }
  };

  const onSplitterPointerMove = (event: React.PointerEvent<HTMLDivElement>) => {
    if (!event.currentTarget.hasPointerCapture(event.pointerId)) {
      return;
    }
    const layout = layoutRef.current;
    if (!layout) {
      return;
    }
    const rect = layout.getBoundingClientRect();
    const next = (event.clientX - rect.left) / rect.width;
    setRatio(Math.min(0.8, Math.max(0.2, next)));
    // The worksheet canvas re-measures only on window resize or re-render, so
    // resizing the panes must force a repaint of both.
    setRevisions((current) => [current[0] + 1, current[1] + 1]);
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", height: 640 }}>
      <div
        style={{ display: "flex", gap: 12, alignItems: "center", padding: 8 }}
      >
        <input
          type="file"
          accept=".xlsx,.csv"
          onChange={(event) => {
            const file = event.target.files?.[0];
            if (file) {
              loadFile(file);
            }
            event.target.value = "";
          }}
        />
        <span style={{ fontFamily: "monospace", fontSize: 12 }}>{status}</span>
      </div>
      <div ref={layoutRef} style={{ display: "flex", flex: 1, minHeight: 0 }}>
        <Pane
          key={`a${generation}`}
          label="session 1"
          model={models[0]}
          revision={revisions[0]}
          offline={offline[0]}
          onOfflineChange={(value) => setOffline((o) => [value, o[1]])}
          grow={ratio}
        />
        <div
          onPointerDown={(event) =>
            event.currentTarget.setPointerCapture(event.pointerId)
          }
          onPointerMove={onSplitterPointerMove}
          style={{
            flex: "0 0 6px",
            cursor: "col-resize",
            touchAction: "none",
            background: "#d0d0d0",
            borderInline: "1px solid #ffffff",
            // The panes hold positioned content, which paints above non-positioned
            // siblings: the splitter must be positioned too to receive the pointer.
            position: "relative",
            zIndex: 10,
          }}
        />
        <Pane
          key={`b${generation}`}
          label="session 2"
          model={models[1]}
          revision={revisions[1]}
          offline={offline[1]}
          onOfflineChange={(value) => setOffline((o) => [o[0], value])}
          grow={1 - ratio}
        />
      </div>
    </div>
  );
}

const meta = {
  title: "Components/Collab",
  component: CollabDemo,
  parameters: {
    layout: "fullscreen",
    docs: {
      description: {
        component: `Two independent workbook widgets backed by two collaborative
models (sessions 1 and 2) that exchange diffs every ${SYNC_INTERVAL_MS}ms.
Load an .xlsx to import it into both sessions, or a .csv to type its cells
into session 1 and watch them sync. Needs the \`make collab\` wasm build.`,
      },
    },
  },
  argTypes: {},
  args: {},
} satisfies Meta<typeof CollabDemo>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {};
