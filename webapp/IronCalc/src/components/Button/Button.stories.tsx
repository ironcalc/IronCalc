import type { Meta, StoryObj } from "@storybook/react";
import {
  Bold,
  ChevronRight,
  Download,
  Italic,
  MoreHorizontal,
  Strikethrough,
  Trash2,
  Underline,
} from "lucide-react";
import { useState } from "react";
import type { ButtonProperties } from "./Button";
import { Button } from "./Button";

const icons = {
  none: undefined,
  bold: <Bold />,
  italic: <Italic />,
  underline: <Underline />,
  strikethrough: <Strikethrough />,
  download: <Download />,
  chevronRight: <ChevronRight />,
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

const defaultArgs: ButtonStoryProps = {
  variant: "primary",
  size: "md",
  iconOnly: false,
  pressed: false,
  startIconName: "none",
  endIconName: "none",
};

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
      options: ["xs", "sm", "md", "lg"],
      description: "Button size",
    },
    iconOnly: {
      control: "boolean",
      description: "Square icon-only button (single icon, no label)",
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
      <Button variant="ghost" endIcon={<ChevronRight />}>
        Ghost
      </Button>
      <Button variant="destructive" startIcon={<Trash2 />}>
        Delete
      </Button>
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
      <Button variant="primary" size="lg">
        Large
      </Button>
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
    iconOnly: true,
    startIconName: "bold",
    pressed: true,
    "aria-label": "Bold",
  },
};

export const FormatToolbar: Story = {
  args: defaultArgs,
  render: function FormatToolbarStory() {
    const [bold, setBold] = useState(true);
    const [italic, setItalic] = useState(false);
    const [underline, setUnderline] = useState(false);
    const [strike, setStrike] = useState(false);

    return (
      <div style={{ display: "flex", gap: 4 }}>
        <Button
          variant="ghost"
          size="sm"
          iconOnly
          startIcon={<Bold />}
          pressed={bold}
          onClick={() => setBold((b) => !b)}
          aria-label="Bold"
        />
        <Button
          variant="ghost"
          size="sm"
          iconOnly
          startIcon={<Italic />}
          pressed={italic}
          onClick={() => setItalic((i) => !i)}
          aria-label="Italic"
        />
        <Button
          variant="ghost"
          size="sm"
          iconOnly
          startIcon={<Underline />}
          pressed={underline}
          onClick={() => setUnderline((u) => !u)}
          aria-label="Underline"
        />
        <Button
          variant="ghost"
          size="sm"
          iconOnly
          startIcon={<Strikethrough />}
          pressed={strike}
          onClick={() => setStrike((s) => !s)}
          aria-label="Strikethrough"
        />
      </div>
    );
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
    endIconName: "chevronRight",
  },
};

export const IconOnly: Story = {
  args: defaultArgs,
  render: () => (
    <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
      <Button
        variant="primary"
        iconOnly
        startIcon={<Download />}
        aria-label="Download"
      />
      <Button
        variant="secondary"
        iconOnly
        startIcon={<MoreHorizontal />}
        aria-label="More"
      />
      <Button variant="ghost" iconOnly startIcon={<Bold />} aria-label="Bold" />
      <Button
        variant="destructive"
        iconOnly
        startIcon={<Trash2 />}
        aria-label="Delete"
      />
    </div>
  ),
};

export const IconOnlySizes: Story = {
  args: defaultArgs,
  render: () => (
    <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
      <Button
        variant="outline"
        size="xs"
        iconOnly
        startIcon={<Download />}
        aria-label="Download"
      />
      <Button
        variant="outline"
        size="sm"
        iconOnly
        startIcon={<Download />}
        aria-label="Download"
      />
      <Button
        variant="outline"
        iconOnly
        startIcon={<Download />}
        aria-label="Download"
      />
      <Button
        variant="outline"
        size="lg"
        iconOnly
        startIcon={<Download />}
        aria-label="Download"
      />
    </div>
  ),
};
