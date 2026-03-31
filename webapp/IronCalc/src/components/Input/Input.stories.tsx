import type { Meta, StoryObj } from "@storybook/react";
import { AlertCircle, Mail, Search } from "lucide-react";
import type { InputProperties } from "./Input";
import { Input } from "./Input";

const adornments = {
  none: undefined,
  search: <Search />,
  mail: <Mail />,
  alertCircle: <AlertCircle />,
} as const;

type AdornmentName = keyof typeof adornments;

type InputStoryProps = Omit<
  InputProperties,
  "startAdornment" | "endAdornment"
> & {
  startAdornmentName?: AdornmentName;
  endAdornmentName?: AdornmentName;
};

function InputStory({
  startAdornmentName = "none",
  endAdornmentName = "none",
  ...props
}: InputStoryProps) {
  return (
    <Input
      {...props}
      startAdornment={adornments[startAdornmentName]}
      endAdornment={adornments[endAdornmentName]}
    />
  );
}

const defaultArgs: InputStoryProps = {};

const meta = {
  title: "Components/Input",
  component: InputStory,
  parameters: {
    layout: "centered",
  },
  args: defaultArgs,
  argTypes: {
    size: {
      control: "select",
      options: ["sm", "md"],
      description: "Input size",
    },
    disabled: {
      control: "boolean",
      description: "Disable the input",
    },
    error: {
      control: "boolean",
      description: "Error state",
    },
    required: {
      control: "boolean",
      description: "Mark field as required (appends * to label)",
    },
    label: {
      control: "text",
      description: "Label rendered above the input",
    },
    helperText: {
      control: "text",
      description: "Helper or error text rendered below the input",
    },
    placeholder: {
      control: "text",
      description: "Placeholder text",
    },
    startAdornmentName: {
      control: "select",
      options: Object.keys(adornments),
      description: "Icon shown on the left",
    },
    endAdornmentName: {
      control: "select",
      options: Object.keys(adornments),
      description: "Icon shown on the right",
    },
  },
} satisfies Meta<typeof InputStory>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    ...defaultArgs,
    label: "Label",
    placeholder: "Placeholder",
    size: "md",
  },
};

export const AllStates: Story = {
  args: defaultArgs,
  render: () => (
    <div
      style={{ display: "flex", flexDirection: "column", gap: 16, width: 280 }}
    >
      <Input placeholder="Default" />
      <Input
        label="Complete"
        value="Some value"
        helperText="Helpful hint goes here."
        required
      />
      <Input
        label="Error"
        value="$$bad"
        onChange={() => {}}
        error
        helperText="This value is invalid."
      />
      <Input
        label="Disabled"
        placeholder="Cannot edit"
        disabled
        helperText="This field is locked."
      />
    </div>
  ),
};

export const Sizes: Story = {
  args: defaultArgs,
  render: () => (
    <div
      style={{ display: "flex", flexDirection: "column", gap: 12, width: 280 }}
    >
      <Input size="sm" placeholder="Small" />
      <Input size="md" placeholder="Medium" />
    </div>
  ),
};

export const WithStartAdornment: Story = {
  args: {
    ...defaultArgs,
    placeholder: "Search…",
    startAdornmentName: "search",
    size: "md",
  },
};

export const WithBothAdornments: Story = {
  args: {
    ...defaultArgs,
    label: "Email",
    placeholder: "you@example.com",
    startAdornmentName: "mail",
    endAdornmentName: "alertCircle",
    size: "md",
  },
};
