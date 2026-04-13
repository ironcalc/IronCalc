import type { Meta, StoryObj } from "@storybook/react";
import { useState } from "react";
import type { SelectOption, SelectProperties } from "./Select";
import { Select } from "./Select";

const fruitOptions = [
  { value: "apple", label: "Apple" },
  { value: "banana", label: "Banana" },
  { value: "durian", label: "Durian" },
  { value: "elderberry", label: "Elderberry" },
  { value: "macadamiaNut", label: "Macadamia Nut" },
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
  options: fruitOptions,
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
    const [value, setValue] = useState("durian");
    return (
      <div style={{ width: 120 }}>
        <Select
          label="Breakfast"
          options={fruitOptions}
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
    const [value3, setValue3] = useState("durian");

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
          value={value3}
          onChange={setValue3}
          error
          helperText="This value is invalid."
        />
        <Select
          label="Disabled"
          options={fruitOptions}
          value="elderberry"
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
    const [value, setValue] = useState("apple");
    return (
      <Select
        variant="ghost"
        options={fruitOptions}
        value={value}
        onChange={setValue}
      />
    );
  },
};

export const CornerPositioning: Story = {
  args: defaultArgs,
  parameters: {
    layout: "fullscreen",
  },
  render: () => {
    const [tl, setTl] = useState("apple");
    const [tr, setTr] = useState("banana");
    const [bl, setBl] = useState("durian");
    const [br, setBr] = useState("elderberry");
    const [lc, setLc] = useState("apple");
    const [tc, setTc] = useState("banana");
    const [rc, setRc] = useState("durian");
    const [bc, setBc] = useState("elderberry");

    return (
      <div>
        <div style={{ position: "absolute", top: 16, left: 16, width: 120 }}>
          <Select
            label="Top-left"
            options={fruitOptions}
            value={tl}
            onChange={setTl}
          />
        </div>

        <div
          style={{
            position: "absolute",
            top: 16,
            left: "50%",
            transform: "translateX(-50%)",
            width: 120,
          }}
        >
          <Select
            label="Top-center"
            options={fruitOptions}
            value={tc}
            onChange={setTc}
          />
        </div>

        <div style={{ position: "absolute", top: 16, right: 16, width: 120 }}>
          <Select
            label="Top-right"
            options={fruitOptions}
            value={tr}
            onChange={setTr}
          />
        </div>

        <div
          style={{
            position: "absolute",
            top: "50%",
            left: 16,
            transform: "translateY(-50%)",
            width: 120,
          }}
        >
          <Select
            label="Left-center"
            options={fruitOptions}
            value={lc}
            onChange={setLc}
          />
        </div>

        <div
          style={{
            position: "absolute",
            top: "50%",
            right: 16,
            transform: "translateY(-50%)",
            width: 120,
          }}
        >
          <Select
            label="Right-center"
            options={fruitOptions}
            value={rc}
            onChange={setRc}
          />
        </div>

        <div style={{ position: "absolute", bottom: 16, left: 16, width: 120 }}>
          <Select
            label="Bottom-left"
            options={fruitOptions}
            value={bl}
            onChange={setBl}
          />
        </div>

        <div
          style={{
            position: "absolute",
            bottom: 16,
            left: "50%",
            transform: "translateX(-50%)",
            width: 120,
          }}
        >
          <Select
            label="Bottom-center"
            options={fruitOptions}
            value={bc}
            onChange={setBc}
          />
        </div>

        <div
          style={{ position: "absolute", bottom: 16, right: 16, width: 120 }}
        >
          <Select
            label="Bottom-right"
            options={fruitOptions}
            value={br}
            onChange={setBr}
          />
        </div>
      </div>
    );
  },
};

export const NoOptions: Story = {
  args: defaultArgs,
  render: () => {
    const [value, setValue] = useState("durian");
    const options: SelectOption[] = [];
    return (
      <div style={{ width: 120 }}>
        <Select
          label="No options"
          options={options}
          value={value}
          onChange={setValue}
        />
      </div>
    );
  },
};

export const Invalid: Story = {
  args: defaultArgs,
  render: () => {
    const [value, setValue] = useState("orange");
    return (
      <div style={{ width: 120 }}>
        <Select
          label="Breakfast"
          options={fruitOptions}
          value={value}
          onChange={setValue}
        />
      </div>
    );
  },
};
