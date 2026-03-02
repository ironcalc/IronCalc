import type { Meta, StoryObj } from "@storybook/react";
import { Mail, MapPin, Search } from "lucide-react";
import { useState } from "react";
import { Button } from "../Button/Button";
import { Input } from "./Input";

const meta = {
  title: "UI/Input",
  component: Input,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
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
    fullWidth: {
      control: "boolean",
      description: "Full width",
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
} satisfies Meta<typeof Input>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    placeholder: "Enter text...",
    size: "md",
    fullWidth: false,
  },
};

export const Ghost: Story = {
  args: {
    placeholder: "Ghost input...",
    variant: "ghost",
    fullWidth: false,
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
  render: () => (
    <div
      style={{ display: "flex", flexDirection: "column", gap: 24, width: 320 }}
    >
      <Input
        label="Email"
        placeholder="Enter email"
        required
        fullWidth={false}
      />
      <Input
        label="Comments"
        placeholder="Enter your comments"
        multiline
        rows={3}
        required
        fullWidth={false}
      />
    </div>
  ),
};

export const WithLabel: Story = {
  render: () => (
    <div
      style={{ display: "flex", flexDirection: "column", gap: 24, width: 320 }}
    >
      <Input label="Name" placeholder="Enter name" fullWidth={false} />
      <Input
        label="Notes"
        placeholder="Enter notes"
        multiline
        rows={3}
        fullWidth={false}
      />
    </div>
  ),
};

export const WithHelperText: Story = {
  render: () => (
    <div
      style={{ display: "flex", flexDirection: "column", gap: 24, width: 320 }}
    >
      <Input
        label="Description"
        placeholder="Enter description"
        helperText="Optional. Add any extra details here."
        fullWidth={false}
      />
      <Input
        label="Comments"
        placeholder="Enter your comments"
        helperText="Optional. Add any extra details here."
        multiline
        rows={3}
        fullWidth={false}
      />
    </div>
  ),
};

export const ErrorState: Story = {
  render: () => (
    <div
      style={{ display: "flex", flexDirection: "column", gap: 24, width: 320 }}
    >
      <Input
        label="Email"
        placeholder="Enter email"
        error
        helperText="This email is already in use."
        defaultValue="user@example.com"
        fullWidth={false}
      />
      <Input
        label="Comments"
        placeholder="Enter your comments"
        multiline
        rows={3}
        error
        helperText="This field is required."
        defaultValue="Some invalid content..."
        fullWidth={false}
      />
    </div>
  ),
};

export const Disabled: Story = {
  render: () => (
    <div
      style={{ display: "flex", flexDirection: "column", gap: 24, width: 320 }}
    >
      <Input
        label="Disabled"
        placeholder="Cannot edit"
        disabled
        defaultValue="Read-only value"
        fullWidth={false}
      />
      <Input
        label="Disabled comments"
        placeholder="Cannot edit"
        multiline
        rows={3}
        disabled
        defaultValue="Read-only multiline content..."
        fullWidth={false}
      />
    </div>
  ),
};

export const Sizes: Story = {
  render: () => (
    <div
      style={{ display: "flex", flexDirection: "column", gap: 16, width: 280 }}
    >
      <Input placeholder="Extra small (xs)" size="xs" fullWidth />
      <Input placeholder="Small (sm)" size="sm" fullWidth />
      <Input placeholder="Medium (md)" size="md" fullWidth />
      <Input placeholder="Large (lg)" size="lg" fullWidth />
    </div>
  ),
};

export const FullWidth: Story = {
  args: {
    label: "Full width input",
    placeholder: "Stretches to container",
    fullWidth: true,
  },
  decorators: [
    (Story) => (
      <div style={{ width: 400 }}>
        <Story />
      </div>
    ),
  ],
};

export const WithButton: Story = {
  render: function WithButtonStory() {
    const [value, setValue] = useState("");
    return (
      <div
        style={{ display: "flex", gap: 8, alignItems: "center", width: 360 }}
      >
        <Input
          placeholder="Enter value..."
          value={value}
          onChange={(e) => setValue(e.target.value)}
          fullWidth
          size="md"
        />
        <Button variant="primary" size="md">
          Submit
        </Button>
      </div>
    );
  },
};

export const Clearable: Story = {
  args: {
    placeholder: "Search...",
    clearable: true,
    fullWidth: false,
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

export const InlineSearchExample: Story = {
  render: function InlineSearchExampleStory() {
    const [query, setQuery] = useState("");
    const [submitted, setSubmitted] = useState<string | null>(null);
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
          variant="ghost"
          placeholder="Search..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          clearable
          startIcon={<Search />}
          onSubmit={() => setSubmitted(query)}
          onCancel={() => {
            setQuery("");
            setSubmitted(null);
          }}
          fullWidth
        />
        {submitted !== null && (
          <span style={{ fontSize: 12, color: "#666" }}>
            Last search: &quot;{submitted}&quot;
          </span>
        )}
      </div>
    );
  },
};

export const FormExample: Story = {
  render: function FormExampleStory() {
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
          label="City"
          placeholder="Optional"
          disabled
          value="Default"
          startIcon={<MapPin size={16} />}
        />
        <Input
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
