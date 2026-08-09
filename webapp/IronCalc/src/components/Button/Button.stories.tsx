import type { Meta, StoryObj } from "@storybook/react";
import {
  Bold,
  ChevronDown,
  Download,
  Italic,
  MoreHorizontal,
  Strikethrough,
  Trash2,
  Underline,
} from "lucide-react";
import type { ButtonProperties } from "./Button";
import { Button } from "./Button";

const icons = {
  none: undefined,
  bold: <Bold />,
  italic: <Italic />,
  underline: <Underline />,
  strikethrough: <Strikethrough />,
  download: <Download />,
  chevronDown: <ChevronDown />,
  moreHorizontal: <MoreHorizontal />,
  trash: <Trash2 />,
} as const;

type IconName = keyof typeof icons;

type ButtonStoryProps = Omit<ButtonProperties, "startIcon" | "endIcon"> & {
  startIconName?: IconName;
  endIconName?: IconName;
};

// Wrapper so Storybook gets a plain function component and nice icon controls.
function ButtonStory({
  startIconName = "none",
  endIconName = "none",
  ...props
}: ButtonStoryProps) {
  return (
    <Button
      {...props}
      startIcon={icons[startIconName]}
      endIcon={icons[endIconName]}
    />
  );
}

const defaultArgs: ButtonStoryProps = {};

const meta = {
  title: "Components/Button",
  component: ButtonStory,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: defaultArgs,
  argTypes: {
    variant: {
      control: "select",
      options: ["primary", "secondary", "outline", "ghost", "destructive"],
      description: "Visual style of the button",
    },
    disabled: {
      control: "boolean",
      description: "Disable the button",
    },
    size: {
      control: "select",
      options: ["xs", "sm", "md"],
      description: "Button size",
    },
    pressed: {
      control: "boolean",
      description: "Toggle state (e.g. Bold/Italic when formatting is on)",
    },
    startIconName: {
      control: "select",
      options: Object.keys(icons),
      description: "Icon shown on the left",
    },
    endIconName: {
      control: "select",
      options: Object.keys(icons),
      description: "Icon shown on the right",
    },
  },
} satisfies Meta<typeof ButtonStory>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    ...defaultArgs,
    children: "Button",
    variant: "primary",
    size: "md",
  },
};

export const Variants: Story = {
  args: defaultArgs,
  render: () => (
    <div
      style={{
        display: "flex",
        gap: 12,
        alignItems: "center",
        flexWrap: "wrap",
      }}
    >
      <Button>Primary</Button>
      <Button variant="secondary">Secondary</Button>
      <Button variant="outline">Outline</Button>
      <Button variant="ghost">Ghost</Button>
      <Button variant="destructive">Delete</Button>
    </div>
  ),
};

export const Sizes: Story = {
  args: defaultArgs,
  render: () => (
    <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
      <Button variant="primary" size="xs">
        Extra small
      </Button>
      <Button variant="primary" size="sm">
        Small
      </Button>
      <Button variant="primary">Medium</Button>
    </div>
  ),
};

export const Disabled: Story = {
  args: {
    ...defaultArgs,
    children: "Disabled",
    variant: "primary",
    disabled: true,
  },
};

export const Pressed: Story = {
  args: {
    ...defaultArgs,
    variant: "ghost",
    size: "sm",
    endIconName: "chevronDown",
    pressed: true,
    children: "A1:A100",
  },
};

export const WithIconLeft: Story = {
  args: {
    ...defaultArgs,
    children: "Download",
    variant: "primary",
    startIconName: "download",
    endIconName: "none",
  },
};

export const WithIconsBoth: Story = {
  args: {
    ...defaultArgs,
    children: "Delete",
    variant: "destructive",
    startIconName: "trash",
    endIconName: "chevronDown",
  },
};
