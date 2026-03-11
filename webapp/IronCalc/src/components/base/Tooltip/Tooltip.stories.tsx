import type { Meta, StoryObj } from "@storybook/react";
import { Bold, Download, Italic, Strikethrough, Underline } from "lucide-react";
import { Button } from "../Button/Button";
import { Tooltip } from "./Tooltip";

const meta = {
  title: "UI/Tooltip",
  component: Tooltip,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  argTypes: {
    title: {
      control: "text",
      description: "Tooltip content",
    },
    placement: {
      control: "select",
      options: [
        "bottom-end",
        "bottom-start",
        "bottom",
        "left-end",
        "left-start",
        "left",
        "right-end",
        "right-start",
        "right",
        "top-end",
        "top-start",
        "top",
      ],
      description: "Tooltip placement relative to the child",
    },
    enterDelay: {
      control: { type: "number", min: 0, max: 2000, step: 100 },
      description: "Delay in ms before showing tooltip",
    },
    leaveDelay: {
      control: { type: "number", min: 0, max: 2000, step: 100 },
      description: "Delay in ms before hiding tooltip",
    },
    disableHoverListener: {
      control: "boolean",
      description: "Disable hover trigger",
    },
    shortcut: {
      control: "text",
      description: "Keyboard shortcut shown after the title, e.g. ⌘B",
    },
  },
} satisfies Meta<typeof Tooltip>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    title: "Tooltip text",
    children: <button type="button">Hover me</button>,
  },
};

export const WithButton: Story = {
  args: {
    title: "Download",
    children: (
      <Button variant="primary" startIcon={<Download />}>
        Download
      </Button>
    ),
  },
};

export const WithIconButton: Story = {
  args: {
    title: "Bold",
    children: (
      <Button iconOnly variant="ghost" startIcon={<Bold />} aria-label="Bold" />
    ),
  },
};

export const WithShortcut: Story = {
  args: {
    title: "Bold",
    shortcut: "⌘B",
    children: (
      <Button iconOnly variant="ghost" startIcon={<Bold />} aria-label="Bold" />
    ),
  },
};

export const Placements: Story = {
  args: {
    title: "",
    children: <span />,
  },
  render: () => (
    <div
      style={{
        display: "flex",
        gap: 24,
        flexWrap: "wrap",
        justifyContent: "center",
        alignItems: "center",
        padding: 48,
      }}
    >
      <Tooltip title="Top" placement="top">
        <Button variant="outline">Top</Button>
      </Tooltip>
      <Tooltip title="Bottom" placement="bottom">
        <Button variant="outline">Bottom</Button>
      </Tooltip>
      <Tooltip title="Left" placement="left">
        <Button variant="outline">Left</Button>
      </Tooltip>
      <Tooltip title="Right" placement="right">
        <Button variant="outline">Right</Button>
      </Tooltip>
    </div>
  ),
};

export const LongContent: Story = {
  args: {
    title:
      "This is a longer tooltip that might wrap onto multiple lines when the content exceeds a reasonable width.",
    children: <Button variant="ghost">Hover for long tooltip</Button>,
  },
};

export const TextFormatIcons: Story = {
  args: {
    title: "",
    children: <span />,
  },
  render: () => (
    <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
      <Tooltip title="Bold" shortcut="⌘B">
        <Button
          iconOnly
          size="xs"
          variant="ghost"
          startIcon={<Bold />}
          aria-label="Bold"
        />
      </Tooltip>
      <Tooltip title="Italic" shortcut="⌘I">
        <Button
          iconOnly
          size="xs"
          variant="ghost"
          startIcon={<Italic />}
          aria-label="Italic"
        />
      </Tooltip>
      <Tooltip title="Underline" shortcut="⌘U">
        <Button
          iconOnly
          size="xs"
          variant="ghost"
          startIcon={<Underline />}
          aria-label="Underline"
        />
      </Tooltip>
      <Tooltip title="Strikethrough" shortcut="⌘⇧X">
        <Button
          iconOnly
          size="xs"
          variant="ghost"
          startIcon={<Strikethrough />}
          aria-label="Strikethrough"
        />
      </Tooltip>
    </div>
  ),
};
