import type { Meta, StoryObj } from "@storybook/react";
import { useEffect, useState } from "react";
import { init, Model } from "../../index";
import { WorkbookState } from "../workbookState";
import Workbook from "./Workbook";

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
        return Model.from_bytes(bytes, "en");
      }
    } catch {
      // Fall through to a new empty workbook on any fetch/parse error.
    }
  }
  return new Model("Workbook1", "en", "UTC", "en");
}

function WorkbookWithInit() {
  const [model, setModel] = useState<Model | null>(null);

  useEffect(() => {
    async function start() {
      await init();
      setModel(await loadModel());
    }
    start();
  }, []);

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
      <Workbook model={model} workbookState={new WorkbookState()} />
    </div>
  );
}

const meta = {
  title: "Components/Workbook",
  component: WorkbookWithInit,
  parameters: {
    layout: "fullscreen",
  },
  argTypes: {},
  args: {},
} satisfies Meta<typeof WorkbookWithInit>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {};
