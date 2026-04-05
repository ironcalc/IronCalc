import type { Meta, StoryObj } from "@storybook/react";
import { useState } from "react";
import type { SelectProperties } from "./Select";
import { Select } from "./Select";

const fruitOptions = [
  { value: "apple", label: "Apple" },
  { value: "banana", label: "Banana" },
  { value: "cherry", label: "Cherry" },
  { value: "durian", label: "Durian" },
  { value: "elderberry", label: "Elderberry" },
];

const countryOptions = [
  { value: "it", label: "Italy" },
  { value: "gb", label: "United Kingdom" },
  { value: "de", label: "Germany" },
  { value: "fr", label: "France" },
  { value: "es", label: "Spain" },
];

type SelectStoryProps = Omit<SelectProperties, "value" | "onChange"> & {
  defaultValue?: string;
};

function SelectStory({
  defaultValue,
  options = fruitOptions,
  ...props
}: SelectStoryProps) {
  const [value, setValue] = useState(defaultValue ?? options[0]?.value ?? "");
  return (
    <Select {...props} options={options} value={value} onChange={setValue} />
  );
}

const defaultArgs: SelectStoryProps = {
  options: countryOptions,
};

const meta = {
  title: "Components/Select",
  component: SelectStory,
  parameters: {
    layout: "centered",
  },
  args: defaultArgs,
  argTypes: {
    size: {
      control: "select",
      options: ["sm", "md"],
      description: "Select size",
    },
    disabled: {
      control: "boolean",
      description: "Disable the select",
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
      description: "Label rendered above the select",
    },
    helperText: {
      control: "text",
      description: "Helper or error text rendered below the select",
    },
    defaultValue: {
      control: "text",
      description: "Initially selected value",
    },
  },
} satisfies Meta<typeof SelectStory>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: defaultArgs,
  render: () => {
    const [value, setValue] = useState("de");
    return (
      <div style={{ width: 160 }}>
        <Select
          label="Country"
          options={countryOptions}
          value={value}
          onChange={setValue}
        />
      </div>
    );
  },
};

export const AllStates: Story = {
  args: defaultArgs,
  render: () => {
    const [value1, setValue1] = useState("apple");
    const [value2, setValue2] = useState("banana");

    return (
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: 16,
          width: 280,
        }}
      >
        <Select options={fruitOptions} value={value1} onChange={setValue1} />
        <Select
          label="Complete"
          options={fruitOptions}
          value={value2}
          onChange={setValue2}
          helperText="Helpful hint goes here."
          required
        />
        <Select
          label="Error"
          options={fruitOptions}
          value="cherry"
          onChange={() => {}}
          error
          helperText="This value is invalid."
        />
        <Select
          label="Disabled"
          options={fruitOptions}
          value="apple"
          onChange={() => {}}
          disabled
          helperText="This field is locked."
        />
      </div>
    );
  },
};

export const Sizes: Story = {
  args: defaultArgs,
  render: () => {
    const [v1, setV1] = useState("apple");
    const [v2, setV2] = useState("apple");

    return (
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: 12,
          width: 280,
        }}
      >
        <Select options={fruitOptions} value={v1} onChange={setV1} size="sm" />
        <Select options={fruitOptions} value={v2} onChange={setV2} size="md" />
      </div>
    );
  },
};

export const Ghost: Story = {
  args: defaultArgs,
  render: () => {
    const [value, setValue] = useState("de");
    return (
      <Select
        variant="ghost"
        size="sm"
        options={countryOptions}
        value={value}
        onChange={setValue}
      />
    );
  },
};
