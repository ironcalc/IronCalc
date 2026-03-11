import type { Meta, StoryObj } from "@storybook/react";
import { Mail, MapPin, Search } from "lucide-react";
import { useState } from "react";
import { Button } from "../Button/Button";
import type { InputProps } from "./Input";
import { Input } from "./Input";

const defaultInputArgs: Partial<InputProps> = {
  variant: "outlined",
  size: "md",
  margin: "none",
  label: undefined,
  clearable: false,
  startIcon: undefined,
  required: false,
  multiline: false,
  rows: undefined,
  error: false,
  helperText: undefined,
  slotProps: undefined,
  sx: undefined,
};

const meta = {
  title: "UI/Input",
  component: Input,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: defaultInputArgs,
  argTypes: {
    size: {
      control: "select",
      options: ["xs", "sm", "md", "lg"],
      description: "Size (aligned with Button sizes)",
    },
    variant: {
      control: "select",
      options: ["outlined", "filled", "standard", "ghost"],
      description: "Visual variant",
    },
    disabled: {
      control: "boolean",
      description: "Disable the input",
    },
    error: {
      control: "boolean",
      description: "Error state",
    },
    multiline: {
      control: "boolean",
      description: "Multiline text area",
    },
    rows: {
      control: { type: "number", min: 1, max: 10 },
      description: "Number of rows (multiline only)",
    },
    placeholder: {
      control: "text",
      description: "Placeholder text",
    },
    label: {
      control: "text",
      description: "Label text",
    },
    helperText: {
      control: "text",
      description: "Helper or error text below input",
    },
    clearable: {
      control: "boolean",
      description: "Show clear (X) button when the input has content",
    },
    startIcon: {
      control: false,
      description: "Icon to show at the start of the input (e.g. Search)",
    },
    required: {
      control: "boolean",
      description: "Shows orange asterisk after label (mandatory field)",
    },
  },
} as Meta<typeof Input>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: { ...defaultInputArgs, placeholder: "Your Name" },
};

export const Ghost: Story = {
  args: {
    ...defaultInputArgs,
    variant: "ghost",
    placeholder: "Search...",
    startIcon: <Search />,
  },
  render: (args) => (
    <div
      style={{ display: "flex", flexDirection: "column", gap: 16, width: 280 }}
    >
      <Input {...args} />
    </div>
  ),
};

export const Required: Story = {
  args: {
    ...defaultInputArgs,
    required: true,
    label: "Email",
    placeholder: "Enter email",
  },
  render: (args) => (
    <div
      style={{ display: "flex", flexDirection: "column", gap: 24, width: 320 }}
    >
      <Input {...args} />
    </div>
  ),
};

export const WithLabel: Story = {
  args: {
    ...defaultInputArgs,
    label: "Name",
    placeholder: "Enter name",
  },
  render: (args) => (
    <div
      style={{ display: "flex", flexDirection: "column", gap: 24, width: 320 }}
    >
      <Input {...args} />
      <Input {...args} multiline={true} placeholder="Enter your comments" />
    </div>
  ),
};

export const WithHelperText: Story = {
  args: {
    ...defaultInputArgs,
    label: "Description",
    placeholder: "Enter description",
    helperText: "Optional. Add any extra details here.",
    rows: 3,
  },
  render: (args) => (
    <div
      style={{ display: "flex", flexDirection: "column", gap: 24, width: 320 }}
    >
      <Input {...args} />
      <Input
        {...args}
        label="Comments"
        multiline={true}
        placeholder="Enter your comments"
      />
    </div>
  ),
};

export const ErrorState: Story = {
  args: defaultInputArgs,
  render: (args) => (
    <div
      style={{ display: "flex", flexDirection: "column", gap: 24, width: 320 }}
    >
      <Input
        {...args}
        label="Email"
        placeholder="Enter email"
        error
        helperText="This email is already in use."
        defaultValue="user@example.com"
      />
      <Input
        {...args}
        label="Comments"
        placeholder="Enter your comments"
        multiline
        rows={3}
        error
        helperText="This field is required."
        defaultValue="Some invalid content..."
      />
    </div>
  ),
};

export const Disabled: Story = {
  args: {
    ...defaultInputArgs,
    label: "Disabled",
    placeholder: "Cannot edit",
    helperText: "Read-only value",
    disabled: true,
  },
  render: (args) => (
    <div
      style={{ display: "flex", flexDirection: "column", gap: 24, width: 320 }}
    >
      <Input {...args} />
    </div>
  ),
};

export const Sizes: Story = {
  args: {
    ...defaultInputArgs,
  },
  render: (args) => (
    <div
      style={{ display: "flex", flexDirection: "column", gap: 16, width: 280 }}
    >
      <Input {...args} placeholder="Extra small (xs)" size="xs" />
      <Input {...args} placeholder="Small (sm)" size="sm" />
      <Input {...args} placeholder="Medium (md)" size="md" />
      <Input {...args} placeholder="Large (lg)" size="lg" />
    </div>
  ),
};

export const WithButton: Story = {
  args: defaultInputArgs,
  render: function WithButtonStory(args) {
    const [value, setValue] = useState("");
    return (
      <div
        style={{ display: "flex", gap: 8, alignItems: "center", width: 360 }}
      >
        <Input
          {...args}
          placeholder="Enter value..."
          value={value}
          onChange={(e) => setValue(e.target.value)}
          size="md"
        />
        <Button
          variant="primary"
          size="md"
          iconOnly={false}
          pressed={false}
          startIcon={undefined}
          endIcon={undefined}
        >
          Submit
        </Button>
      </div>
    );
  },
};

export const Clearable: Story = {
  args: {
    ...defaultInputArgs,
    placeholder: "Search...",
    clearable: true,
  },
  render: function ClearableStory(args) {
    const [value, setValue] = useState("");
    return (
      <div style={{ width: 280 }}>
        <Input
          {...args}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          startIcon={<Search />}
        />
      </div>
    );
  },
};

export const GhostSearchExample: Story = {
  args: {
    ...defaultInputArgs,
    size: "md",
    variant: "ghost",
    clearable: true,
    startIcon: <Search />,
    placeholder: "Search...",
  },
  render: function GhostSearchExampleStory(args) {
    const [query, setQuery] = useState("");
    return (
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: 12,
          width: 320,
        }}
      >
        <Input
          {...args}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
      </div>
    );
  },
};

export const FormExample: Story = {
  args: defaultInputArgs,
  render: function FormExampleStory(args) {
    const [name, setName] = useState("");
    const [notes, setNotes] = useState("");
    const [nameError, setNameError] = useState("");

    const validateName = (value: string) => {
      if (value.length > 0 && value.length < 3) {
        setNameError("Name must be at least 3 characters");
      } else {
        setNameError("");
      }
    };

    return (
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: 16,
          width: 320,
        }}
      >
        <Input
          {...args}
          label="Email"
          placeholder="Enter email"
          value={name}
          onChange={(e) => {
            setName(e.target.value);
            validateName(e.target.value);
          }}
          error={!!nameError}
          helperText={nameError}
          startIcon={<Mail />}
        />
        <Input
          {...args}
          label="City"
          placeholder="Optional"
          disabled
          value="Default"
          startIcon={<MapPin size={16} />}
        />
        <Input
          {...args}
          label="Notes"
          placeholder="Enter notes"
          multiline
          rows={3}
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
        />
      </div>
    );
  },
};
