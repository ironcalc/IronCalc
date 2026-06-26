import type { Meta, StoryObj } from "@storybook/react";
import { useEffect, useRef, useState } from "react";
import { init, Model } from "../../index";
import { FormulaHelper } from "./FormulaHelper";
import { applyListCompletion, getCompletion } from "./formulaCompletion";

// Standalone playground for the Formula Helper. It is intentionally not wired
// into the real editor: a plain <input> drives it. Type a formula such as:
//   =SU        -> list of matching function names (↑/↓ to move, Enter to pick)
//   =SUMIF(    -> SUMIF signature and argument docs
function FormulaHelperPlayground() {
  const [model, setModel] = useState<Model | null>(null);
  const [text, setText] = useState("=SU");
  const [cursor, setCursor] = useState(3);
  const [selected, setSelected] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    async function start() {
      await init();
      setModel(new Model("Workbook1", "en", "UTC", "en"));
    }
    start();
  }, []);

  if (!model) {
    return <div>Loading…</div>;
  }

  // Re-read the cursor after the browser applies the change/selection.
  const syncCursor = () => {
    setCursor(inputRef.current?.selectionStart ?? text.length);
  };

  const completion = text.startsWith("=")
    ? getCompletion(model, text, cursor)
    : null;

  // Accept the highlighted function: replace the partial name with `NAME(`.
  const accept = () => {
    const input = inputRef.current;
    if (!input || completion?.kind !== "list") {
      return;
    }
    const result = applyListCompletion(text, cursor, completion, selected);
    setText(result.text);
    setSelected(0);
    requestAnimationFrame(() => {
      input.setSelectionRange(result.cursor, result.cursor);
      setCursor(result.cursor);
    });
  };

  return (
    <div style={{ padding: 40, width: 520 }}>
      <p style={{ color: "#6a6a6a", fontSize: 13 }}>
        Type a formula, e.g. <code>=SU</code> or <code>=SUMIF(</code>. Use ↑/↓
        and Enter in the list.
      </p>
      <div style={{ position: "relative" }}>
        <input
          ref={inputRef}
          value={text}
          spellCheck={false}
          onChange={(event) => {
            setText(event.target.value);
            setSelected(0);
            syncCursor();
          }}
          onKeyUp={syncCursor}
          onClick={syncCursor}
          onKeyDown={(event) => {
            if (completion?.kind !== "list") {
              return;
            }
            if (event.key === "ArrowDown") {
              event.preventDefault();
              setSelected((value) =>
                Math.min(value + 1, completion.matches.length - 1),
              );
            } else if (event.key === "ArrowUp") {
              event.preventDefault();
              setSelected((value) => Math.max(value - 1, 0));
            } else if (event.key === "Enter" || event.key === "Tab") {
              event.preventDefault();
              accept();
            }
          }}
          style={{
            width: "100%",
            boxSizing: "border-box",
            padding: "8px 10px",
            fontSize: 15,
            fontFamily: "ui-monospace, Menlo, Consolas, monospace",
            border: "2px solid #f5a623",
            borderRadius: 4,
            outline: "none",
          }}
        />
        {completion ? (
          <div
            style={{ position: "absolute", top: "100%", left: 0, marginTop: 4 }}
          >
            <FormulaHelper
              completion={completion}
              selected={selected}
              onSelect={setSelected}
            />
          </div>
        ) : null}
      </div>
    </div>
  );
}

const meta = {
  title: "Components/FormulaHelper",
  component: FormulaHelperPlayground,
  parameters: {
    layout: "fullscreen",
  },
} satisfies Meta<typeof FormulaHelperPlayground>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Playground: Story = {};
