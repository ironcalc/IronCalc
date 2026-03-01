import type { Meta, StoryObj } from "@storybook/react";
import {
  Bold,
  ChevronRight,
  Download,
  MoreHorizontal,
  Trash2,
} from "lucide-react";
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

export const Primary: Story = {
  args: {
    children: "Primary",
    variant: "primary",
  },
};

export const Secondary: Story = {
  args: {
    children: "Secondary",
    variant: "secondary",
  },
};

export const Outline: Story = {
  args: {
    children: "Outline",
    variant: "outline",
  },
};

export const Ghost: Story = {
  args: {
    children: "Ghost",
    variant: "ghost",
    endIcon: <ChevronRight />,
  },
};

export const Destructive: Story = {
  args: {
    children: "Delete",
    variant: "destructive",
    startIcon: <Trash2 />,
  },
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

export const DisabledDestructive: Story = {
  args: {
    children: "Delete",
    variant: "destructive",
    disabled: true,
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
        variant="primary"
        startIcon={<Download />}
        aria-label="Download"
      />
      <Button
        iconOnly
        size="sm"
        variant="primary"
        startIcon={<Download />}
        aria-label="Download"
      />
      <Button
        iconOnly
        size="md"
        variant="primary"
        startIcon={<Download />}
        aria-label="Download"
      />
      <Button
        iconOnly
        size="lg"
        variant="primary"
        startIcon={<Download />}
        aria-label="Download"
      />
    </div>
  ),
};
