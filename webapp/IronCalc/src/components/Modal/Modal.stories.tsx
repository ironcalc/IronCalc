import type { Meta, StoryObj } from "@storybook/react";
import { useState } from "react";
import { Button } from "../Button/Button";
import { Alert } from "./Alert";
import { Confirm } from "./Confirm";
import { Prompt } from "./Prompt";

const meta = {
  title: "Components/Modal",
  parameters: { layout: "centered" },
} satisfies Meta;

export default meta;

type Story = StoryObj;

export const AlertStory: Story = {
  name: "Alert",
  render: () => {
    const [open, setOpen] = useState(false);
    return (
      <>
        <Button variant="secondary" onClick={() => setOpen(true)}>
          Open alert
        </Button>
        <Alert
          open={open}
          onClose={() => setOpen(false)}
          title="Something happened"
          message="This is an informational message."
        />
      </>
    );
  },
};

export const ConfirmStory: Story = {
  name: "Confirm",
  render: () => {
    const [openDefault, setOpenDefault] = useState(false);
    const [openDestructive, setOpenDestructive] = useState(false);
    return (
      <>
        <div style={{ display: "flex", flexDirection: "row", gap: 8 }}>
          <Button variant="secondary" onClick={() => setOpenDefault(true)}>
            Confirm
          </Button>
          <Button
            variant="destructive"
            onClick={() => setOpenDestructive(true)}
          >
            Destructive
          </Button>
        </div>
        <Confirm
          open={openDefault}
          onClose={() => setOpenDefault(false)}
          onConfirm={() => {}}
          title="Confirm action"
          message="Are you sure you want to proceed?"
        />
        <Confirm
          open={openDestructive}
          onClose={() => setOpenDestructive(false)}
          onConfirm={() => {}}
          title="Delete item"
          message="Are you sure you want to delete this item? This action cannot be undone."
          confirmLabel="Yes, delete"
          variant="destructive"
        />
      </>
    );
  },
};

export const PromptStory: Story = {
  name: "Prompt",
  render: () => {
    const [open, setOpen] = useState(false);
    return (
      <>
        <Button variant="secondary" onClick={() => setOpen(true)}>
          Open prompt
        </Button>
        <Prompt
          open={open}
          onClose={() => setOpen(false)}
          onSubmit={() => {}}
          title="Enter a value"
          defaultValue="default"
          confirmLabel="Save"
        />
      </>
    );
  },
};
