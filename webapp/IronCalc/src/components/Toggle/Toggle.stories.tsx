import type { Meta, StoryObj } from "@storybook/react";
import { useState } from "react";
import type { ToggleProperties } from "./Toggle";
import { Toggle } from "./Toggle";

type ToggleStoryProps = Omit<ToggleProperties, "checked" | "onChange"> & {
  checked?: boolean;
};

// Wrapper so the toggle is controlled by the story, not by the caller.
function ToggleStory({ checked = false, ...props }: ToggleStoryProps) {
  const [value, setValue] = useState(checked);
  return <Toggle {...props} checked={value} onChange={setValue} />;
}

const defaultArgs: ToggleStoryProps = {};

const meta = {
  title: "Components/Toggle",
  component: ToggleStory,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: defaultArgs,
  argTypes: {
    checked: {
      control: "boolean",
      description: "Initial state of the toggle",
    },
    disabled: {
      control: "boolean",
      description: "Disable the toggle",
    },
    label: {
      control: "text",
      description: "Optional label rendered on the left of the switch",
    },
  },
} satisfies Meta<typeof ToggleStory>;

export default meta;

type Story = StoryObj<typeof meta>;

const Column = ({ children }: { children: React.ReactNode }) => (
  <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
    {children}
  </div>
);

export const Default: Story = {
  args: defaultArgs,
  render: () => (
    <Column>
      <ToggleStory checked />
      <ToggleStory />
    </Column>
  ),
};

export const WithLabel: Story = {
  args: defaultArgs,
  render: () => (
    <Column>
      <ToggleStory label="On" checked />
      <ToggleStory label="Off" />
    </Column>
  ),
};

export const Disabled: Story = {
  args: defaultArgs,
  render: () => (
    <Column>
      <ToggleStory label="On" checked disabled />
      <ToggleStory label="Off" disabled />
    </Column>
  ),
};
