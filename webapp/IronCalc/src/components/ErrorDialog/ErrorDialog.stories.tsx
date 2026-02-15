import type { Meta, StoryObj } from "@storybook/react";
import { useState } from "react";
import { Button } from "../Button/Button";
import ErrorDialog from "./ErrorDialog";

const meta = {
  title: "Components/ErrorDialog",
  component: ErrorDialog,
  parameters: {
    layout: "centered",
  },
} satisfies Meta<typeof ErrorDialog>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    open: true,
    onClose: () => {},
    title: "Failed to delete cells",
  },
  render: () => {
    const [open, setOpen] = useState(true);
    return (
      <>
        <Button onClick={() => setOpen(true)}>Open dialog</Button>
        <ErrorDialog
          open={open}
          onClose={() => setOpen(false)}
          title="Failed to delete cells"
        />
      </>
    );
  },
};
