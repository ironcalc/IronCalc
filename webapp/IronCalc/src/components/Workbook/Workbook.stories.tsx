import type { Meta, StoryObj } from "@storybook/react";
import { useEffect, useState } from "react";
import { init, Model } from "../../index";
import { WorkbookState } from "../workbookState";
import Workbook from "./Workbook";

function WorkbookWithInit() {
  const [model, setModel] = useState<Model | null>(null);

  useEffect(() => {
    async function start() {
      await init();
      setModel(new Model("Workbook1", "en", "UTC", "en"));
    }
    start();
  }, []);

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
      }}
    >
      <Workbook model={model} workbookState={new WorkbookState()} />
    </div>
  );
}

const meta = {
  title: "Example/Workbook",
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
