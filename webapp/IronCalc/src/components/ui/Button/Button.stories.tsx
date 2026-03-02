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
import { Button } from "./Button";

const meta = {
  title: "UI/Button",
  component: Button,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
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
    children: "Button",
    variant: "primary",
    size: "md",
  },
};

export const Variants: Story = {
  render: () => (
    <div
      style={{
        display: "flex",
        gap: 12,
        alignItems: "center",
        flexWrap: "wrap",
      }}
    >
      <Button variant="primary">Primary</Button>
      <Button variant="secondary">Secondary</Button>
      <Button variant="outline">Outline</Button>
      <Button variant="ghost" endIcon={<ChevronRight />}>
        Ghost
      </Button>
      <Button variant="destructive" startIcon={<Trash2 />}>
        Delete
      </Button>
    </div>
  ),
};

export const Sizes: Story = {
  render: () => (
    <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
      <Button size="xs">Extra small</Button>
      <Button size="sm">Small</Button>
      <Button size="md">Medium</Button>
      <Button size="lg">Large</Button>
    </div>
  ),
};

export const Disabled: Story = {
  args: {
    children: "Disabled",
    variant: "primary",
    disabled: true,
  },
};

export const Pressed: Story = {
  args: {
    children: "Bold",
    variant: "ghost",
    size: "sm",
    iconOnly: true,
    startIcon: <Bold />,
    pressed: true,
    "aria-label": "Bold",
  },
};

export const FormatToolbar: Story = {
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
          iconOnly
          startIcon={<Bold />}
          pressed={bold}
          onClick={() => setBold((b) => !b)}
          aria-label="Bold"
        />
        <Button
          variant="ghost"
          size="sm"
          iconOnly
          startIcon={<Italic />}
          pressed={italic}
          onClick={() => setItalic((i) => !i)}
          aria-label="Italic"
        />
        <Button
          variant="ghost"
          size="sm"
          iconOnly
          startIcon={<Underline />}
          pressed={underline}
          onClick={() => setUnderline((u) => !u)}
          aria-label="Underline"
        />
        <Button
          variant="ghost"
          size="sm"
          iconOnly
          startIcon={<Strikethrough />}
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
    children: "Download",
    variant: "primary",
    startIcon: <Download />,
  },
};

export const WithIconsBoth: Story = {
  args: {
    children: "Delete",
    variant: "destructive",
    startIcon: <Trash2 />,
    endIcon: <ChevronRight />,
  },
};

export const IconOnly: Story = {
  render: () => (
    <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
      <Button
        iconOnly
        variant="primary"
        startIcon={<Download />}
        aria-label="Download"
      />
      <Button
        iconOnly
        variant="secondary"
        startIcon={<MoreHorizontal />}
        aria-label="More"
      />
      <Button iconOnly variant="ghost" startIcon={<Bold />} aria-label="Bold" />
      <Button
        iconOnly
        variant="destructive"
        startIcon={<Trash2 />}
        aria-label="Delete"
      />
    </div>
  ),
};

export const IconOnlySizes: Story = {
  render: () => (
    <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
      <Button
        iconOnly
        size="xs"
        variant="outline"
        startIcon={<Download />}
        aria-label="Download"
      />
      <Button
        iconOnly
        size="sm"
        variant="outline"
        startIcon={<Download />}
        aria-label="Download"
      />
      <Button
        iconOnly
        size="md"
        variant="outline"
        startIcon={<Download />}
        aria-label="Download"
      />
      <Button
        iconOnly
        size="lg"
        variant="outline"
        startIcon={<Download />}
        aria-label="Download"
      />
    </div>
  ),
};
