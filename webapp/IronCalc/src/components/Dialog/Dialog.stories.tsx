import type { Meta, StoryObj } from "@storybook/react";
import { useState } from "react";
import { Button } from "../Button/Button";
import { Dialog } from "./Dialog";

const meta = {
  title: "Components/Dialog",
  component: Dialog,
  parameters: {
    layout: "centered",
  },
  argTypes: {
    title: {
      control: "text",
      description: "Dialog title",
    },
    open: { table: { disable: true } },
    onClose: { table: { disable: true } },
    children: { table: { disable: true } },
    footer: { table: { disable: true } },
  },
} satisfies Meta<typeof Dialog>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: (args) => {
    const [open, setOpen] = useState(false);
    return (
      <>
        <Button variant="secondary" onClick={() => setOpen(true)}>
          Open dialog
        </Button>
        <Dialog
          {...args}
          open={open}
          onClose={() => setOpen(false)}
          showHeader
          footer={
            <>
              <Button
                size="md"
                variant="secondary"
                onClick={() => setOpen(false)}
              >
                Cancel
              </Button>
              <Button size="md" onClick={() => setOpen(false)}>
                Confirm
              </Button>
            </>
          }
        >
          <p style={{ margin: 0 }}>
            This is the dialog body. You can put any content here.
          </p>
        </Dialog>
      </>
    );
  },
  args: {
    open: false,
    onClose: () => {},
    title: "Dialog title",
    children: null,
  },
};

export const Destructive: Story = {
  render: () => {
    const [open, setOpen] = useState(false);
    return (
      <>
        <Button variant="destructive" onClick={() => setOpen(true)}>
          Delete item
        </Button>
        <Dialog
          open={open}
          onClose={() => setOpen(false)}
          onConfirm={() => setOpen(false)}
          title="Delete item"
          showHeader
          footer={
            <>
              <Button
                size="md"
                variant="secondary"
                onClick={() => setOpen(false)}
              >
                Cancel
              </Button>
              <Button
                size="md"
                variant="destructive"
                onClick={() => setOpen(false)}
              >
                Yes, delete
              </Button>
            </>
          }
        >
          <p>
            Are you sure you want to delete this item? This action cannot be
            undone.
          </p>
        </Dialog>
      </>
    );
  },
  args: { open: false, onClose: () => {}, children: null },
};

export const NoHeader: Story = {
  render: () => {
    const [open, setOpen] = useState(false);
    return (
      <>
        <Button variant="secondary" onClick={() => setOpen(true)}>
          Open dialog
        </Button>
        <Dialog
          open={open}
          onClose={() => setOpen(false)}
          footer={
            <>
              <Button
                size="md"
                variant="secondary"
                onClick={() => setOpen(false)}
              >
                Cancel
              </Button>
              <Button size="md" onClick={() => setOpen(false)}>
                OK
              </Button>
            </>
          }
        >
          <p style={{ margin: 0 }}>
            A dialog without a header. Closes on Esc or backdrop click.
          </p>
        </Dialog>
      </>
    );
  },
  args: { open: false, onClose: () => {}, children: null },
};
