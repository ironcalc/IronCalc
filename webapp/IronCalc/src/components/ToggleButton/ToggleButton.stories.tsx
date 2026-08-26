import type { Meta, StoryObj } from "@storybook/react";
import { AlignCenter, AlignLeft, AlignRight } from "lucide-react";
import { useState } from "react";
import type { ToggleButtonProperties } from "./ToggleButton";
import { ToggleButton } from "./ToggleButton";

type ToggleButtonStoryProps = Omit<
  ToggleButtonProperties<string>,
  "value" | "onChange"
> & {
  value?: string;
};

// Wrapper so the selection is controlled by the story, not by the caller.
function ToggleButtonStory({
  value,
  options,
  ...props
}: ToggleButtonStoryProps) {
  const [selected, setSelected] = useState(value ?? options[0].value);
  return (
    <ToggleButton
      {...props}
      options={options}
      value={selected}
      onChange={setSelected}
    />
  );
}

const modeOptions = [
  { value: "preset", label: "Presets" },
  { value: "ratings", label: "Ratings" },
];

const operatorOptions = [
  { value: ">=", icon: "≥", "aria-label": "Greater than or equal" },
  { value: ">", icon: ">", "aria-label": "Greater than" },
];

const alignOptions = [
  { value: "left", icon: <AlignLeft />, "aria-label": "Align left" },
  { value: "center", icon: <AlignCenter />, "aria-label": "Align center" },
  { value: "right", icon: <AlignRight />, "aria-label": "Align right" },
];

const defaultArgs: ToggleButtonStoryProps = { options: modeOptions };

const meta = {
  title: "Components/ToggleButton",
  component: ToggleButtonStory,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: defaultArgs,
  argTypes: {
    size: {
      control: "select",
      options: ["xs", "sm", "md"],
      description: "Size of the buttons",
    },
    fullWidth: {
      control: "boolean",
      description: "Stretch the group and share its width between the options",
    },
    disabled: {
      control: "boolean",
      description: "Disable every option",
    },
  },
} satisfies Meta<typeof ToggleButtonStory>;

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
      <ToggleButtonStory options={modeOptions} />
      <ToggleButtonStory options={operatorOptions} />
      <ToggleButtonStory options={alignOptions} />
    </Column>
  ),
};

export const Sizes: Story = {
  args: defaultArgs,
  render: () => (
    <Column>
      <ToggleButtonStory options={alignOptions} size="xs" />
      <ToggleButtonStory options={alignOptions} size="sm" />
      <ToggleButtonStory options={alignOptions} size="md" />
    </Column>
  ),
};

export const FullWidth: Story = {
  args: defaultArgs,
  render: () => (
    <div style={{ width: 260 }}>
      <ToggleButtonStory options={modeOptions} fullWidth />
    </div>
  ),
};

export const Disabled: Story = {
  args: defaultArgs,
  render: () => (
    <Column>
      <ToggleButtonStory options={modeOptions} disabled />
      <ToggleButtonStory
        options={[
          { value: "preset", label: "Presets" },
          { value: "ratings", label: "Ratings", disabled: true },
        ]}
      />
    </Column>
  ),
};
