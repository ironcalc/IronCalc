import type { Meta, StoryObj } from "@storybook/react";
import {
  Bold,
  Download,
  Italic,
  MoreHorizontal,
  Strikethrough,
  Trash2,
  Underline,
} from "lucide-react";
import { useState } from "react";
import type { IconButtonProperties } from "./IconButton";
import { IconButton } from "./IconButton";

const icons = {
  bold: <Bold />,
  italic: <Italic />,
  underline: <Underline />,
  strikethrough: <Strikethrough />,
  download: <Download />,
  moreHorizontal: <MoreHorizontal />,
  trash: <Trash2 />,
} as const;

type IconName = keyof typeof icons;

type IconButtonStoryProps = Omit<IconButtonProperties, "icon"> & {
  iconName?: IconName;
};

function IconButtonStory({
  iconName = "download",
  ...props
}: IconButtonStoryProps) {
  return (
    <IconButton
      {...props}
      icon={icons[iconName]}
      aria-label={props["aria-label"] ?? iconName}
    />
  );
}

const defaultArgs: IconButtonStoryProps = {
  variant: "primary",
  size: "md",
  pressed: false,
  iconName: "download",
  "aria-label": "Download",
};

const meta = {
  title: "Components/IconButton",
  component: IconButtonStory,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: defaultArgs,
  argTypes: {
    variant: {
      control: "select",
      options: ["primary", "secondary", "outline", "ghost", "destructive"],
      description: "Visual style",
    },
    size: {
      control: "select",
      options: ["xs", "sm", "md"],
      description: "Button size",
    },
    disabled: {
      control: "boolean",
      description: "Disable the button",
    },
    pressed: {
      control: "boolean",
      description: "Toggle / selected state",
    },
    iconName: {
      control: "select",
      options: Object.keys(icons),
      description: "Icon to show",
    },
    "aria-label": {
      control: "text",
      description: "Accessible label (required for icon-only buttons)",
    },
  },
} satisfies Meta<typeof IconButtonStory>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    ...defaultArgs,
    variant: "primary",
    size: "md",
    "aria-label": "Download",
  },
};

export const Variants: Story = {
  render: () => (
    <div
      style={{
        display: "flex",
        gap: 12,
        alignItems: "center",
        flexWrap: "wrap",
      }}
    >
      <IconButton variant="primary" icon={<Download />} aria-label="Download" />
      <IconButton
        variant="secondary"
        icon={<MoreHorizontal />}
        aria-label="More"
      />
      <IconButton variant="outline" icon={<Download />} aria-label="Download" />
      <IconButton variant="ghost" icon={<Bold />} aria-label="Bold" />
      <IconButton variant="destructive" icon={<Trash2 />} aria-label="Delete" />
    </div>
  ),
};

export const Sizes: Story = {
  render: () => (
    <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
      <IconButton
        variant="outline"
        size="xs"
        icon={<Download />}
        aria-label="Download"
      />
      <IconButton
        variant="outline"
        size="sm"
        icon={<Download />}
        aria-label="Download"
      />
      <IconButton variant="outline" icon={<Download />} aria-label="Download" />
    </div>
  ),
};

export const Disabled: Story = {
  args: {
    ...defaultArgs,
    variant: "primary",
    disabled: true,
    "aria-label": "Download",
  },
};

export const Pressed: Story = {
  args: {
    ...defaultArgs,
    variant: "ghost",
    size: "sm",
    pressed: true,
    iconName: "bold",
    "aria-label": "Bold",
  },
};

export const FormatToolbar: Story = {
  render: function FormatToolbarStory() {
    const [bold, setBold] = useState(true);
    const [italic, setItalic] = useState(false);
    const [underline, setUnderline] = useState(false);
    const [strike, setStrike] = useState(false);

    return (
      <div style={{ display: "flex", gap: 4 }}>
        <IconButton
          variant="ghost"
          size="sm"
          icon={<Bold />}
          pressed={bold}
          onClick={() => setBold((b) => !b)}
          aria-label="Bold"
        />
        <IconButton
          variant="ghost"
          size="sm"
          icon={<Italic />}
          pressed={italic}
          onClick={() => setItalic((i) => !i)}
          aria-label="Italic"
        />
        <IconButton
          variant="ghost"
          size="sm"
          icon={<Underline />}
          pressed={underline}
          onClick={() => setUnderline((u) => !u)}
          aria-label="Underline"
        />
        <IconButton
          variant="ghost"
          size="sm"
          icon={<Strikethrough />}
          pressed={strike}
          onClick={() => setStrike((s) => !s)}
          aria-label="Strikethrough"
        />
      </div>
    );
  },
};
