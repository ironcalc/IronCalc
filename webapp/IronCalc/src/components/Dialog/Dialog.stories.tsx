import type { Meta, StoryObj } from "@storybook/react";
import { Check, Trash2 } from "lucide-react";
import { useState } from "react";
import { Button } from "../Button/Button";
import { Dialog, type DialogProperties } from "./Dialog";

type DialogStoryArgs = DialogProperties & {
  showFooter: boolean;
};

const meta = {
  title: "Components/Dialog",
  component: Dialog,
  parameters: {
    layout: "centered",
  },
  args: {
    title: "Dialog title",
    showCloseButton: true,
    showFooter: true,
    width: 300,
  },
  argTypes: {
    title: {
      control: "text",
    },
    showCloseButton: {
      control: "boolean",
    },
    showFooter: {
      control: "boolean",
    },
    width: {
      control: "number",
    },
  },
} satisfies Meta<DialogStoryArgs>;

export default meta;

type Story = StoryObj<DialogStoryArgs>;

export const Default: Story = {
  render: (args) => {
    const [open, setOpen] = useState(true);
    const { showFooter, ...dialogArgs } = args;

    return (
      <>
        <Dialog
          {...dialogArgs}
          open={open}
          onClose={() => setOpen(false)}
          footer={
            showFooter ? (
              <>
                <Button variant="secondary" onClick={() => setOpen(false)}>
                  Cancel
                </Button>
                <Button
                  variant="primary"
                  startIcon={<Check />}
                  onClick={() => setOpen(false)}
                >
                  Confirm
                </Button>
              </>
            ) : undefined
          }
        >
          <div>Dialog content</div>
        </Dialog>

        <div
          style={{
            minHeight: "100vh",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          <Button variant="outline" onClick={() => setOpen(true)}>
            Open dialog
          </Button>
        </div>
      </>
    );
  },
};

export const Confirmation: Story = {
  args: {
    title: "Delete sheet",
    showCloseButton: false,
    showFooter: true,
  },
  render: (args) => {
    const [open, setOpen] = useState(true);
    const { showFooter, ...dialogArgs } = args;

    return (
      <>
        <Dialog
          {...dialogArgs}
          open={open}
          onClose={() => setOpen(false)}
          footer={
            showFooter ? (
              <>
                <Button variant="secondary" onClick={() => setOpen(false)}>
                  Cancel
                </Button>
                <Button
                  variant="destructive"
                  startIcon={<Trash2 />}
                  onClick={() => setOpen(false)}
                >
                  Delete
                </Button>
              </>
            ) : undefined
          }
        >
          <div>
            Are you sure you want to delete the sheet <b>"Sheet1"</b>? This
            action cannot be undone.
          </div>
        </Dialog>

        <div
          style={{
            minHeight: "100vh",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          <Button variant="outline" onClick={() => setOpen(true)}>
            Open dialog
          </Button>
        </div>
      </>
    );
  },
};

export const LongText: Story = {
  args: {
    title: "The Metamorphosis",
    showFooter: true,
  },
  render: (args) => {
    const [open, setOpen] = useState(true);
    const { showFooter, ...dialogArgs } = args;

    const longText = `One morning, when Gregor Samsa woke from troubled dreams, he found himself transformed in his bed into a horrible vermin. He lay on his armour-like back, and if he lifted his head a little he could see his brown belly, slightly domed and divided by arches into stiff sections. The bedding was hardly able to cover it and seemed ready to slide off any moment. His many legs, pitifully thin compared with the size of the rest of him, waved about helplessly as he looked. "What's happened to me?" he thought. It wasn't a dream. His room, a proper human room although a little too small, lay peacefully between its four familiar walls. A collection of textile samples lay spread out on the table - Samsa was a travelling salesman - and above it there hung a picture that he had recently cut out of an illustrated magazine and housed in a nice, gilded frame. It showed a lady fitted out with a fur hat and fur boa who sat upright, raising a heavy fur muff that covered the whole of her lower arm towards the viewer. Gregor then turned to look out the window at the dull weather."`;

    return (
      <>
        <Dialog
          {...dialogArgs}
          open={open}
          onClose={() => setOpen(false)}
          footer={
            showFooter ? (
              <>
                <Button variant="secondary" onClick={() => setOpen(false)}>
                  Cancel
                </Button>
                <Button variant="primary" onClick={() => setOpen(false)}>
                  Understood
                </Button>
              </>
            ) : undefined
          }
        >
          <p style={{ margin: 0, whiteSpace: "pre-wrap", lineHeight: 1.5 }}>
            {longText}
          </p>
        </Dialog>

        <div
          style={{
            minHeight: "100vh",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          <Button variant="outline" onClick={() => setOpen(true)}>
            Open dialog
          </Button>
        </div>
      </>
    );
  },
};
