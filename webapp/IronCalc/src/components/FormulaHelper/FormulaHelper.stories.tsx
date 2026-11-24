import type { Meta, StoryObj } from "@storybook/react";
import { useEffect, useRef, useState } from "react";
import { init, Model } from "../../index";
import { FormulaHelper } from "./FormulaHelper";
import { applyListCompletion, getCompletion } from "./formulaCompletion";

// Standalone playground for the Formula Helper. It is intentionally not wired
// into the real editor: a plain <input> drives it. Type a formula such as:
//   =SU        -> list of matching function names (↑/↓ to move, Enter to pick)
//   =SUMIF(    -> SUMIF signature and argument docs
//   =SUR       -> a named function and a defined name, each with its tag
function FormulaHelperPlayground() {
  const [model, setModel] = useState<Model | null>(null);
  const [text, setText] = useState("=SU");
  const [cursor, setCursor] = useState(3);
  const [selected, setSelected] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    async function start() {
      await init();
      const workbook = new Model("Workbook1", "en", "UTC", "en");
      // Something of the user's own, to see next to the built-in functions.
      workbook.newNamedFunction({
        name: "Surcharge",
        formula: "amount*(1+rate)",
        description: "Adds a surcharge to an amount.",
        parameters: [
          { name: "amount", description: "The amount before the surcharge" },
          {
            name: "rate",
            description: "The surcharge, as a fraction of the amount",
            optional: true,
          },
        ],
      });
      workbook.newDefinedName({
        name: "SurchargeRate",
        formula: "Sheet1!$A$1",
        comment: "The surcharge applied to every order",
      });
      setModel(workbook);
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
  const accept = (index: number = selected) => {
    const input = inputRef.current;
    if (!input || completion?.kind !== "list") {
      return;
    }
    const result = applyListCompletion(text, cursor, completion, index);
    setText(result.text);
    setSelected(0);
    requestAnimationFrame(() => {
      input.setSelectionRange(result.cursor, result.cursor);
      setCursor(result.cursor);
    });
  };

  return (
    <div style={{ width: 320 }}>
      <p
        style={{
          color: "var(--palette-grey-700)",
          fontSize: "var(--typography-font-size)",
          fontFamily: "var(--typography-font-family)",
        }}
      >
        Type a formula, e.g. <code>=SU</code> or <code>=SUMIF(</code>.<br />
        Use ↑/↓ and Enter in the list.
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
            width: 140,
            boxSizing: "border-box",
            padding: "8px",
            fontSize: "var(--typography-font-size)",
            fontFamily: "var(--typography-font-family)",
            background: "var(--palette-common-white)",
            color: "var(--palette-common-black)",
            border: "2px solid var(--palette-primary-main)",
            outline:
              "3px solid color-mix(in srgb, var(--palette-primary-main) 20%, transparent)",
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
              onAccept={accept}
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
    layout: "centered",
  },
} satisfies Meta<typeof FormulaHelperPlayground>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {};
