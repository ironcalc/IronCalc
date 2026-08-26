import type { Meta, StoryObj } from "@storybook/react";
import { useState } from "react";
import type { CheckboxProperties } from "./Checkbox";
import { Checkbox } from "./Checkbox";

type CheckboxStoryProps = Omit<CheckboxProperties, "checked" | "onChange"> & {
  checked?: boolean;
};

// Wrapper so the checkbox is controlled by the story, not by the caller.
function CheckboxStory({ checked = false, ...props }: CheckboxStoryProps) {
  const [value, setValue] = useState(checked);
  return <Checkbox {...props} checked={value} onChange={setValue} />;
}

const defaultArgs: CheckboxStoryProps = {};

const meta = {
  title: "Components/Checkbox",
  component: CheckboxStory,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: defaultArgs,
  argTypes: {
    checked: {
      control: "boolean",
      description: "Initial state of the checkbox",
    },
    disabled: {
      control: "boolean",
      description: "Disable the checkbox",
    },
    label: {
      control: "text",
      description: "Optional label rendered on the right of the checkbox",
    },
  },
} satisfies Meta<typeof CheckboxStory>;

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
      <CheckboxStory checked />
      <CheckboxStory />
    </Column>
  ),
};

export const WithLabel: Story = {
  args: defaultArgs,
  render: () => (
    <Column>
      <CheckboxStory label="Checked" checked />
      <CheckboxStory label="Unchecked" />
    </Column>
  ),
};

export const Disabled: Story = {
  args: defaultArgs,
  render: () => (
    <Column>
      <CheckboxStory label="Checked" checked disabled />
      <CheckboxStory label="Unchecked" disabled />
    </Column>
  ),
};
