import type { Meta, StoryObj } from "@storybook/react";
import type { TextareaProperties } from "./Textarea";
import { Textarea } from "./Textarea";

const defaultArgs: TextareaProperties = {};

const meta = {
  title: "Components/Textarea",
  component: Textarea,
  parameters: {
    layout: "centered",
  },
  args: defaultArgs,
  argTypes: {
    size: {
      control: "select",
      options: ["sm", "md"],
      description: "Textarea size",
    },
    disabled: {
      control: "boolean",
      description: "Disable the textarea",
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
      description: "Label rendered above the textarea",
    },
    helperText: {
      control: "text",
      description: "Helper or error text rendered below the textarea",
    },
    placeholder: {
      control: "text",
      description: "Placeholder text",
    },
    rows: {
      control: "number",
      description: "Number of visible text lines",
    },
  },
} satisfies Meta<typeof Textarea>;

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
      <Textarea placeholder="Default" />
      <Textarea
        label="Complete"
        defaultValue="Some value"
        helperText="Helpful hint goes here."
        required
      />
      <Textarea
        label="Error"
        defaultValue="$$bad"
        error
        helperText="This value is invalid."
      />
      <Textarea
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
      <Textarea size="sm" placeholder="Small" />
      <Textarea size="md" placeholder="Medium" />
    </div>
  ),
};

export const WithRows: Story = {
  args: {
    ...defaultArgs,
    label: "Description",
    placeholder: "Write something…",
    rows: 6,
    size: "md",
  },
};
