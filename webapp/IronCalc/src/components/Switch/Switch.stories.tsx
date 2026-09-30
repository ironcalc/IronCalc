import type { Meta, StoryObj } from "@storybook/react";
import { useState } from "react";
import { useArgs } from "storybook/preview-api";
import type { SwitchProperties } from "./Switch";
import { Switch } from "./Switch";

type SwitchStoryProps = Omit<SwitchProperties, "checked" | "onChange"> & {
  checked?: boolean;
};

// Wrapper so the switch is controlled by the story, not by the caller.
function SwitchStory({ checked = false, ...props }: SwitchStoryProps) {
  const [value, setValue] = useState(checked);
  return <Switch {...props} checked={value} onChange={setValue} />;
}

const meta = {
  title: "Components/Switch",
  component: SwitchStory,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: { checked: false, disabled: false, label: "" },
  argTypes: {
    checked: {
      control: "boolean",
      description: "State of the switch",
    },
    disabled: {
      control: "boolean",
      description: "Disable the switch",
    },
    label: {
      control: "text",
      description: "Optional label rendered on the left of the switch",
    },
  },
} satisfies Meta<typeof SwitchStory>;

export default meta;

type Story = StoryObj<typeof meta>;

const Column = ({ children }: { children: React.ReactNode }) => (
  <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
    {children}
  </div>
);

export const Default: Story = {
  render: function DefaultStory({ checked = false, ...args }) {
    const [, updateArgs] = useArgs();
    return (
      <Switch
        {...args}
        checked={checked}
        onChange={(value) => updateArgs({ checked: value })}
      />
    );
  },
};

export const WithLabel: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <Column>
      <SwitchStory label="On" checked />
      <SwitchStory label="Off" />
    </Column>
  ),
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <Column>
      <SwitchStory label="On" checked disabled />
      <SwitchStory label="Off" disabled />
    </Column>
  ),
};
