import type { Meta, StoryObj } from "@storybook/react";
import {
  Bold,
  ChevronRight,
  Download,
  Italic,
  MoreHorizontal,
  Strikethrough,
  Trash2,
  Underline,
} from "lucide-react";
import { useState } from "react";
import type { ButtonProps } from "./Button";
import { Button } from "./Button";

// Wrapper so Storybook receives a function component (forwardRef components are objects)
function ButtonStory(props: ButtonProps) {
  return <Button {...props} />;
}

const defaultArgs = {
  variant: "primary" as const,
  size: "md" as const,
  iconOnly: false,
  pressed: false,
  startIcon: undefined as React.ReactNode,
  endIcon: undefined as React.ReactNode,
};

const meta = {
  title: "UI/Button",
  component: ButtonStory,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: defaultArgs,
  argTypes: {
    variant: {
      control: "select",
      options: ["primary", "secondary", "outline", "ghost", "destructive"],
      description: "Visual style of the button",
    },
    disabled: {
      control: "boolean",
      description: "Disable the button",
    },
    size: {
      control: "select",
      options: ["xs", "sm", "md", "lg"],
      description: "Button size",
    },
    iconOnly: {
      control: "boolean",
      description: "Square icon-only button (single icon, no label)",
    },
    pressed: {
      control: "boolean",
      description: "Toggle state (e.g. Bold/Italic when formatting is on)",
    },
  },
} satisfies Meta<typeof Button>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    ...defaultArgs,
    children: "Button",
    variant: "primary",
    size: "md",
  },
};

export const Variants: Story = {
  args: defaultArgs,
  render: () => (
    <div
      style={{
        display: "flex",
        gap: 12,
        alignItems: "center",
        flexWrap: "wrap",
      }}
    >
      <Button
        variant="primary"
        size="md"
        iconOnly={false}
        pressed={false}
        startIcon={undefined}
        endIcon={undefined}
      >
        Primary
      </Button>
      <Button
        variant="secondary"
        size="md"
        iconOnly={false}
        pressed={false}
        startIcon={undefined}
        endIcon={undefined}
      >
        Secondary
      </Button>
      <Button
        variant="outline"
        size="md"
        iconOnly={false}
        pressed={false}
        startIcon={undefined}
        endIcon={undefined}
      >
        Outline
      </Button>
      <Button
        variant="ghost"
        size="md"
        iconOnly={false}
        pressed={false}
        startIcon={undefined}
        endIcon={<ChevronRight />}
      >
        Ghost
      </Button>
      <Button
        variant="destructive"
        size="md"
        iconOnly={false}
        pressed={false}
        startIcon={<Trash2 />}
        endIcon={undefined}
      >
        Delete
      </Button>
    </div>
  ),
};

export const Sizes: Story = {
  args: defaultArgs,
  render: () => (
    <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
      <Button
        variant="primary"
        size="xs"
        iconOnly={false}
        pressed={false}
        startIcon={undefined}
        endIcon={undefined}
      >
        Extra small
      </Button>
      <Button
        variant="primary"
        size="sm"
        iconOnly={false}
        pressed={false}
        startIcon={undefined}
        endIcon={undefined}
      >
        Small
      </Button>
      <Button
        variant="primary"
        size="md"
        iconOnly={false}
        pressed={false}
        startIcon={undefined}
        endIcon={undefined}
      >
        Medium
      </Button>
      <Button
        variant="primary"
        size="lg"
        iconOnly={false}
        pressed={false}
        startIcon={undefined}
        endIcon={undefined}
      >
        Large
      </Button>
    </div>
  ),
};

export const Disabled: Story = {
  args: {
    ...defaultArgs,
    children: "Disabled",
    variant: "primary",
    disabled: true,
  },
};

export const Pressed: Story = {
  args: {
    ...defaultArgs,
    children: "Bold",
    variant: "ghost",
    size: "sm",
    iconOnly: true,
    startIcon: <Bold />,
    endIcon: undefined,
    pressed: true,
    "aria-label": "Bold",
  },
};

export const FormatToolbar: Story = {
  args: defaultArgs,
  render: function FormatToolbarStory() {
    const [bold, setBold] = useState(true);
    const [italic, setItalic] = useState(false);
    const [underline, setUnderline] = useState(false);
    const [strike, setStrike] = useState(false);
    return (
      <div style={{ display: "flex", gap: 4 }}>
        <Button
          variant="ghost"
          size="sm"
          iconOnly={true}
          startIcon={<Bold />}
          endIcon={undefined}
          pressed={bold}
          onClick={() => setBold((b) => !b)}
          aria-label="Bold"
        />
        <Button
          variant="ghost"
          size="sm"
          iconOnly={true}
          startIcon={<Italic />}
          endIcon={undefined}
          pressed={italic}
          onClick={() => setItalic((i) => !i)}
          aria-label="Italic"
        />
        <Button
          variant="ghost"
          size="sm"
          iconOnly={true}
          startIcon={<Underline />}
          endIcon={undefined}
          pressed={underline}
          onClick={() => setUnderline((u) => !u)}
          aria-label="Underline"
        />
        <Button
          variant="ghost"
          size="sm"
          iconOnly={true}
          startIcon={<Strikethrough />}
          endIcon={undefined}
          pressed={strike}
          onClick={() => setStrike((s) => !s)}
          aria-label="Strikethrough"
        />
      </div>
    );
  },
};

export const WithIconLeft: Story = {
  args: {
    ...defaultArgs,
    children: "Download",
    variant: "primary",
    startIcon: <Download />,
    endIcon: undefined,
  },
};

export const WithIconsBoth: Story = {
  args: {
    ...defaultArgs,
    children: "Delete",
    variant: "destructive",
    startIcon: <Trash2 />,
    endIcon: <ChevronRight />,
  },
};

export const IconOnly: Story = {
  args: defaultArgs,
  render: () => (
    <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
      <Button
        variant="primary"
        size="md"
        iconOnly={true}
        pressed={false}
        startIcon={<Download />}
        endIcon={undefined}
        aria-label="Download"
      />
      <Button
        variant="secondary"
        size="md"
        iconOnly={true}
        pressed={false}
        startIcon={<MoreHorizontal />}
        endIcon={undefined}
        aria-label="More"
      />
      <Button
        variant="ghost"
        size="md"
        iconOnly={true}
        pressed={false}
        startIcon={<Bold />}
        endIcon={undefined}
        aria-label="Bold"
      />
      <Button
        variant="destructive"
        size="md"
        iconOnly={true}
        pressed={false}
        startIcon={<Trash2 />}
        endIcon={undefined}
        aria-label="Delete"
      />
    </div>
  ),
};

export const IconOnlySizes: Story = {
  args: defaultArgs,
  render: () => (
    <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
      <Button
        variant="outline"
        size="xs"
        iconOnly={true}
        pressed={false}
        startIcon={<Download />}
        endIcon={undefined}
        aria-label="Download"
      />
      <Button
        variant="outline"
        size="sm"
        iconOnly={true}
        pressed={false}
        startIcon={<Download />}
        endIcon={undefined}
        aria-label="Download"
      />
      <Button
        variant="outline"
        size="md"
        iconOnly={true}
        pressed={false}
        startIcon={<Download />}
        endIcon={undefined}
        aria-label="Download"
      />
      <Button
        variant="outline"
        size="lg"
        iconOnly={true}
        pressed={false}
        startIcon={<Download />}
        endIcon={undefined}
        aria-label="Download"
      />
    </div>
  ),
};
