import type { Meta, StoryObj } from "@storybook/react";
import {
  AlignCenter,
  AlignLeft,
  AlignRight,
  Bold,
  Italic,
  Strikethrough,
  Underline,
} from "lucide-react";
import { useState } from "react";
import {
  BorderBottomIcon,
  BorderLeftIcon,
  BorderRightIcon,
  BorderTopIcon,
} from "../../icons";
import type {
  ToggleButtonOption,
  ToggleButtonProperties,
} from "./ToggleButton";
import { ToggleButton } from "./ToggleButton";

type ToggleButtonStoryProps = Omit<
  ToggleButtonProperties<string>,
  "value" | "onChange" | "multiple"
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

// Every option toggles on its own, so the selection is a list.
function MultipleStory({ options, ...props }: ToggleButtonStoryProps) {
  const [selected, setSelected] = useState<string[]>([]);
  return (
    <ToggleButton
      {...props}
      multiple
      options={options}
      value={selected}
      onChange={setSelected}
    />
  );
}

// A lone option toggles on and off: the empty value matches no option.
function SingleOptionStory({ option }: { option: ToggleButtonOption<string> }) {
  const [value, setValue] = useState(option.value);
  return (
    <ToggleButton
      options={[option]}
      value={value}
      onChange={(next) => setValue(value === next ? "" : next)}
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

const fontOptions = [
  { value: "bold", icon: <Bold />, "aria-label": "Bold" },
  { value: "italic", icon: <Italic />, "aria-label": "Italic" },
  { value: "underline", icon: <Underline />, "aria-label": "Underline" },
  {
    value: "strikethrough",
    icon: <Strikethrough />,
    "aria-label": "Strikethrough",
  },
];

const borderOptions = [
  { value: "top", icon: <BorderTopIcon />, "aria-label": "Top border" },
  { value: "right", icon: <BorderRightIcon />, "aria-label": "Right border" },
  {
    value: "bottom",
    icon: <BorderBottomIcon />,
    "aria-label": "Bottom border",
  },
  { value: "left", icon: <BorderLeftIcon />, "aria-label": "Left border" },
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

// Adjacent selected options round only the outer ends of the run.
export const Multiple: Story = {
  args: defaultArgs,
  render: () => (
    <Column>
      <MultipleStory options={fontOptions} />
      <MultipleStory options={borderOptions} size="md" />
    </Column>
  ),
};

export const SingleOption: Story = {
  args: defaultArgs,
  render: () => (
    <Column>
      <SingleOptionStory option={{ value: "bold", label: "Bold" }} />
      <SingleOptionStory
        option={{ value: "bold", icon: <Bold />, "aria-label": "Bold" }}
      />
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
