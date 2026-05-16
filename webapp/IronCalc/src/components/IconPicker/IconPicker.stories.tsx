import type { Meta, StoryObj } from "@storybook/react";
import { useState } from "react";
import IconPicker from "./IconPicker";

const ICON_NAMES = [
  "ArrowUp",
  "ArrowAngleUp",
  "ArrowRight",
  "ArrowAngleDown",
  "ArrowDown",
  "TriangleUp",
  "FlatRectangle",
  "TriangleDown",
  "Circle",
  "Rhombus",
  "Flag",
  "Check",
  "Cross",
  "Exclamation",
  "Star",
  "Heart",
  "ThumbsUp",
  "ThumbsDown",
] as const;

type IconName = (typeof ICON_NAMES)[number];

interface IconPickerStoryProps {
  value: IconName;
  color: string;
}

const labelStyle: React.CSSProperties = {
  fontFamily: "monospace",
  fontSize: 12,
  color: "var(--palette-grey-600)",
  minWidth: 100,
};

function IconPickerStory({ value: initialValue, color }: IconPickerStoryProps) {
  const [value, setValue] = useState<string>(initialValue);
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 8, padding: 16 }}>
      <div style={{ width: 32, height: 28 }}>
        <IconPicker value={value} color={color} onChange={setValue} />
      </div>
      <span style={labelStyle}>{value}</span>
    </div>
  );
}

const meta = {
  title: "Components/IconPicker",
  component: IconPickerStory,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: {
    value: "Star",
    color: "#FFD700",
  },
  argTypes: {
    value: {
      control: "select",
      options: ICON_NAMES,
      description: "Currently selected icon (backend name)",
    },
    color: {
      control: "color",
      description: "Icon color",
    },
  },
} satisfies Meta<typeof IconPickerStory>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const MultipleColors: Story = {
  render: function MultipleColorsStory() {
    const [icon1, setIcon1] = useState("ArrowUp");
    const [icon2, setIcon2] = useState("Check");
    const [icon3, setIcon3] = useState("Star");

    const rows: [string, string, (v: string) => void][] = [
      ["#8CB354", icon1, setIcon1],
      ["#F8CD3D", icon2, setIcon2],
      ["#EC5753", icon3, setIcon3],
    ];

    return (
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: 8,
          padding: 16,
        }}
      >
        {rows.map(([color, value, onChange]) => (
          <div
            key={color}
            style={{ display: "flex", alignItems: "center", gap: 8 }}
          >
            <div
              style={{
                width: 10,
                height: 10,
                borderRadius: "50%",
                background: color,
                flexShrink: 0,
              }}
            />
            <div style={{ width: 32, height: 28 }}>
              <IconPicker value={value} color={color} onChange={onChange} />
            </div>
            <span style={labelStyle}>{value}</span>
          </div>
        ))}
      </div>
    );
  },
};

export const AllIcons: Story = {
  render: function AllIconsStory() {
    const [selected, setSelected] = useState("Star");
    return (
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: 16,
          padding: 16,
        }}
      >
        <div style={{ display: "flex", flexWrap: "wrap", gap: 8 }}>
          {ICON_NAMES.map((name) => (
            <div
              key={name}
              style={{
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                gap: 4,
              }}
            >
              <div style={{ width: 32, height: 28 }}>
                <IconPicker
                  value={name}
                  color={selected === name ? "#2196F3" : "#666666"}
                  onChange={setSelected}
                />
              </div>
              <span
                style={{
                  fontFamily: "monospace",
                  fontSize: 9,
                  color: "var(--palette-grey-500)",
                }}
              >
                {name}
              </span>
            </div>
          ))}
        </div>
        <div style={{ fontSize: 12, color: "var(--palette-grey-600)" }}>
          Selected: <strong>{selected}</strong>
        </div>
      </div>
    );
  },
};
