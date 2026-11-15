import type { Meta, StoryObj } from "@storybook/react";
import { useState } from "react";
import { Radio } from "./Radio";

type RadioGroupStoryProps = {
  disabled?: boolean;
  withDescriptions?: boolean;
};

const options = [
  { value: "first", label: "First option", description: "The default." },
  { value: "second", label: "Second option", description: "An alternative." },
  { value: "third", label: "Third option", description: "Another one." },
];

// Wrapper so the group is controlled by the story, not by the caller.
function RadioGroupStory({
  disabled = false,
  withDescriptions = true,
}: RadioGroupStoryProps) {
  const [value, setValue] = useState("first");
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      {options.map((option) => (
        <Radio
          key={option.value}
          name="radio-story"
          label={option.label}
          description={withDescriptions ? option.description : undefined}
          checked={value === option.value}
          disabled={disabled}
          onChange={() => setValue(option.value)}
        />
      ))}
    </div>
  );
}

const defaultArgs: RadioGroupStoryProps = {};

const meta = {
  title: "Components/Radio",
  component: RadioGroupStory,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: defaultArgs,
  argTypes: {
    disabled: {
      control: "boolean",
      description: "Disable the radio buttons",
    },
    withDescriptions: {
      control: "boolean",
      description: "Render a description under each label",
    },
  },
} satisfies Meta<typeof RadioGroupStory>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: defaultArgs,
};

export const WithoutDescriptions: Story = {
  args: { withDescriptions: false },
};

export const Disabled: Story = {
  args: { disabled: true },
};
