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
    disableHoverListener: {
      control: "boolean",
      description: "Disable hover trigger",
    },
  },
} satisfies Meta<typeof Tooltip>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    title: "Tooltip text",
    placement: "bottom",
    disableHoverListener: false,
    children: <button type="button">Hover me</button>,
  },
};

export const WithButton: Story = {
  args: {
    title: "Download",
    placement: "bottom",
    disableHoverListener: false,
    children: (
      <Button
        variant="primary"
        size="md"
        iconOnly={false}
        pressed={false}
        startIcon={<Download />}
        endIcon={undefined}
      >
        Download
      </Button>
    ),
  },
};

export const WithIconButton: Story = {
  args: {
    title: "Bold",
    placement: "bottom",
    disableHoverListener: false,
    children: (
      <Button
        variant="ghost"
        size="md"
        iconOnly={true}
        pressed={false}
        startIcon={<Bold />}
        endIcon={undefined}
        aria-label="Bold"
      />
    ),
  },
};

export const Placements: Story = {
  args: {
    title: "",
    placement: "bottom",
    disableHoverListener: false,
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
      <Tooltip title="Top" placement="top" disableHoverListener={false}>
        <Button
          variant="outline"
          size="md"
          iconOnly={false}
          pressed={false}
          startIcon={undefined}
          endIcon={undefined}
        >
          Top
        </Button>
      </Tooltip>
      <Tooltip title="Bottom" placement="bottom" disableHoverListener={false}>
        <Button
          variant="outline"
          size="md"
          iconOnly={false}
          pressed={false}
          startIcon={undefined}
          endIcon={undefined}
        >
          Bottom
        </Button>
      </Tooltip>
      <Tooltip title="Left" placement="left" disableHoverListener={false}>
        <Button
          variant="outline"
          size="md"
          iconOnly={false}
          pressed={false}
          startIcon={undefined}
          endIcon={undefined}
        >
          Left
        </Button>
      </Tooltip>
      <Tooltip title="Right" placement="right" disableHoverListener={false}>
        <Button
          variant="outline"
          size="md"
          iconOnly={false}
          pressed={false}
          startIcon={undefined}
          endIcon={undefined}
        >
          Right
        </Button>
      </Tooltip>
    </div>
  ),
};

export const LongContent: Story = {
  args: {
    title:
      "This is a longer tooltip that might wrap onto multiple lines when the content exceeds a reasonable width.",
    placement: "bottom",
    disableHoverListener: false,
    children: (
      <Button
        variant="ghost"
        size="md"
        iconOnly={false}
        pressed={false}
        startIcon={undefined}
        endIcon={undefined}
      >
        Hover for long tooltip
      </Button>
    ),
  },
};

export const TextFormatIcons: Story = {
  args: {
    title: "",
    placement: "bottom",
    disableHoverListener: false,
    children: <span />,
  },
  render: () => (
    <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
      <Tooltip title="Bold" placement="bottom" disableHoverListener={false}>
        <Button
          variant="ghost"
          size="xs"
          iconOnly={true}
          pressed={false}
          startIcon={<Bold />}
          endIcon={undefined}
          aria-label="Bold"
        />
      </Tooltip>
      <Tooltip title="Italic" placement="bottom" disableHoverListener={false}>
        <Button
          variant="ghost"
          size="xs"
          iconOnly={true}
          pressed={false}
          startIcon={<Italic />}
          endIcon={undefined}
          aria-label="Italic"
        />
      </Tooltip>
      <Tooltip
        title="Underline"
        placement="bottom"
        disableHoverListener={false}
      >
        <Button
          variant="ghost"
          size="xs"
          iconOnly={true}
          pressed={false}
          startIcon={<Underline />}
          endIcon={undefined}
          aria-label="Underline"
        />
      </Tooltip>
      <Tooltip
        title="Strikethrough"
        placement="bottom"
        disableHoverListener={false}
      >
        <Button
          variant="ghost"
          size="xs"
          iconOnly={true}
          pressed={false}
          startIcon={<Strikethrough />}
          endIcon={undefined}
          aria-label="Strikethrough"
        />
      </Tooltip>
    </div>
  ),
};
