import type { Meta, StoryObj } from "@storybook/react";
import type { ReactNode } from "react";
import { Button } from "../Button/Button";
import { Tooltip, type TooltipProperties } from "./Tooltip";

function Btn({ children }: { children: ReactNode }) {
  return <Button variant="secondary">{children}</Button>;
}

type TooltipStoryProps = Omit<TooltipProperties, "children">;

function TooltipStory({ title }: TooltipStoryProps) {
  return (
    <Tooltip title={title}>
      <Btn>Hover me</Btn>
    </Tooltip>
  );
}

const meta = {
  title: "Components/Tooltip",
  component: TooltipStory,
  parameters: {
    layout: "centered",
  },
  argTypes: {
    title: {
      control: "text",
      description: "Tooltip text content",
    },
  },
} satisfies Meta<typeof TooltipStory>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    title: "This is a tooltip",
  },
};

export const CornerPositioning: Story = {
  parameters: {
    layout: "fullscreen",
  },
  args: {
    title: "Tooltip text content",
  },
  render: ({ title }) => (
    <div>
      <div style={{ position: "absolute", top: 8, left: 8 }}>
        <Tooltip title={title}>
          <Btn>Top-left</Btn>
        </Tooltip>
      </div>

      <div
        style={{
          position: "absolute",
          top: 8,
          left: "50%",
          transform: "translateX(-50%)",
        }}
      >
        <Tooltip title={title}>
          <Btn>Top-center</Btn>
        </Tooltip>
      </div>

      <div style={{ position: "absolute", top: 8, right: 8 }}>
        <Tooltip title={title}>
          <Btn>Top-right</Btn>
        </Tooltip>
      </div>

      <div
        style={{
          position: "absolute",
          top: "50%",
          left: 8,
          transform: "translateY(-50%)",
        }}
      >
        <Tooltip title={title}>
          <Btn>Left-center</Btn>
        </Tooltip>
      </div>

      <div
        style={{
          position: "absolute",
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -50%)",
        }}
      >
        <Tooltip title={title}>
          <Btn>Center</Btn>
        </Tooltip>
      </div>

      <div
        style={{
          position: "absolute",
          top: "50%",
          right: 8,
          transform: "translateY(-50%)",
        }}
      >
        <Tooltip title={title}>
          <Btn>Right-center</Btn>
        </Tooltip>
      </div>

      <div style={{ position: "absolute", bottom: 8, left: 8 }}>
        <Tooltip title={title}>
          <Btn>Bottom-left</Btn>
        </Tooltip>
      </div>

      <div
        style={{
          position: "absolute",
          bottom: 8,
          left: "50%",
          transform: "translateX(-50%)",
        }}
      >
        <Tooltip title={title}>
          <Btn>Bottom-center</Btn>
        </Tooltip>
      </div>

      <div style={{ position: "absolute", bottom: 8, right: 8 }}>
        <Tooltip title={title}>
          <Btn>Bottom-right</Btn>
        </Tooltip>
      </div>
    </div>
  ),
};
