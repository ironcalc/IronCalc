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

function CollabDemo() {
  const [models, setModels] = useState<[Model, Model] | null>(null);
  const [revisions, setRevisions] = useState<[number, number]>([0, 0]);
  const [offline, setOffline] = useState<[boolean, boolean]>([false, false]);
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
      const a = new Model("demo", "en", "UTC", "en", 1);
      const b = Model.from_bytes(a.toBytes(), 2);
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
        to.applyExternalDiffs(diffs);
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
    <div ref={layoutRef} style={{ display: "flex", height: 600 }}>
      <Pane
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
        label="session 2"
        model={models[1]}
        revision={revisions[1]}
        offline={offline[1]}
        onOfflineChange={(value) => setOffline((o) => [o[0], value])}
        grow={1 - ratio}
      />
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
models (sessions 1 and 2) that exchange diffs every ${SYNC_INTERVAL_MS}ms.`,
      },
    },
  },
  argTypes: {},
  args: {},
} satisfies Meta<typeof CollabDemo>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {};
