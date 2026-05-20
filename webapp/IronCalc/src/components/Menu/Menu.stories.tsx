import type { Meta, StoryObj } from "@storybook/react";
import { Check, Clipboard, Copy, Scissors, Trash2 } from "lucide-react";
import { useContext, useState } from "react";
import { Button } from "../Button/Button";
import { Menu, MenuContext } from "./Menu";
import { MenuDivider } from "./MenuDivider";
import { MenuItem, MenuItemWithSubmenu } from "./MenuItem";

const meta = {
  title: "Components/Menu",
  component: Menu,
  parameters: {
    layout: "centered",
  },
} satisfies Meta<typeof Menu>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    trigger: <Button variant="secondary">Open menu</Button>,
    children: null,
  },
  render: () => (
    <Menu trigger={<Button variant="secondary">Open menu</Button>}>
      <MenuItems />
    </Menu>
  ),
};

function MenuItems() {
  return (
    <>
      <MenuItem icon={<Copy />} secondaryText="⌘C" onClick={() => {}}>
        Copy
      </MenuItem>
      <MenuItem icon={<Scissors />} secondaryText="⌘X" onClick={() => {}}>
        Cut
      </MenuItem>
      <MenuItem icon={<Clipboard />} secondaryText="⌘V" onClick={() => {}}>
        Paste
      </MenuItem>
      <MenuDivider />
      <MenuItem
        icon={<Trash2 />}
        secondaryText="⌘⌫"
        destructive
        onClick={() => {}}
      >
        Delete
      </MenuItem>
    </>
  );
}

function MenuItemsWithSub() {
  return (
    <>
      <MenuItem icon={<Copy />} secondaryText="⌘C" onClick={() => {}}>
        Copy
      </MenuItem>
      <MenuItem icon={<Scissors />} secondaryText="⌘X" onClick={() => {}}>
        Cut
      </MenuItem>
      <MenuItem icon={<Clipboard />} secondaryText="⌘V" onClick={() => {}}>
        Paste
      </MenuItem>
      <MenuDivider />
      <MenuItem
        icon={<Trash2 />}
        secondaryText="⌘⌫"
        destructive
        onClick={() => {}}
      >
        Delete
      </MenuItem>
      <MenuDivider />
      <MenuItemWithSubmenu
        submenu={
          <>
            <MenuItem onClick={() => {}}>Option A</MenuItem>
            <MenuItem onClick={() => {}}>Option B</MenuItem>
            <MenuItem onClick={() => {}}>Option C</MenuItem>
          </>
        }
      >
        More
      </MenuItemWithSubmenu>
    </>
  );
}

export const CornerPositioning: Story = {
  args: {
    trigger: <Button variant="secondary">Open menu</Button>,
    children: null,
  },
  parameters: { layout: "fullscreen" },
  render: () => (
    <div>
      <div style={{ position: "absolute", top: 16, left: 16 }}>
        <Menu trigger={<Button variant="secondary">Top-left</Button>}>
          <MenuItemsWithSub />
        </Menu>
      </div>

      <div
        style={{
          position: "absolute",
          top: 16,
          left: "50%",
          transform: "translateX(-50%)",
        }}
      >
        <Menu trigger={<Button variant="secondary">Top-center</Button>}>
          <MenuItemsWithSub />
        </Menu>
      </div>

      <div style={{ position: "absolute", top: 16, right: 16 }}>
        <Menu trigger={<Button variant="secondary">Top-right</Button>}>
          <MenuItemsWithSub />
        </Menu>
      </div>

      <div
        style={{
          position: "absolute",
          top: "50%",
          left: 16,
          transform: "translateY(-50%)",
        }}
      >
        <Menu trigger={<Button variant="secondary">Left-center</Button>}>
          <MenuItemsWithSub />
        </Menu>
      </div>

      <div
        style={{
          position: "absolute",
          top: "50%",
          right: 16,
          transform: "translateY(-50%)",
        }}
      >
        <Menu trigger={<Button variant="secondary">Right-center</Button>}>
          <MenuItemsWithSub />
        </Menu>
      </div>

      <div style={{ position: "absolute", bottom: 16, left: 16 }}>
        <Menu trigger={<Button variant="secondary">Bottom-left</Button>}>
          <MenuItemsWithSub />
        </Menu>
      </div>

      <div
        style={{
          position: "absolute",
          bottom: 16,
          left: "50%",
          transform: "translateX(-50%)",
        }}
      >
        <Menu trigger={<Button variant="secondary">Bottom-center</Button>}>
          <MenuItemsWithSub />
        </Menu>
      </div>

      <div style={{ position: "absolute", bottom: 16, right: 16 }}>
        <Menu trigger={<Button variant="secondary">Bottom-right</Button>}>
          <MenuItemsWithSub />
        </Menu>
      </div>
    </div>
  ),
};

const ALIGNMENTS = ["Pinapple", "Grapefruit", "Mango", "Durian"] as const;
type Alignment = (typeof ALIGNMENTS)[number];

function AlignmentItems({
  value,
  onChange,
}: {
  value: Alignment;
  onChange: (v: Alignment) => void;
}) {
  const menu = useContext(MenuContext);

  return (
    <>
      {ALIGNMENTS.map((alignment) => {
        const selected = alignment === value;
        return (
          <button
            key={alignment}
            type="button"
            role="menuitemradio"
            aria-checked={selected}
            className="ic-menu-item"
            style={{ gap: 8 }}
            onClick={() => {
              onChange(alignment);
              menu?.close();
            }}
          >
            <Check
              size={16}
              style={{
                flexShrink: 0,
                visibility: selected ? "visible" : "hidden",
              }}
            />
            {alignment}
          </button>
        );
      })}
    </>
  );
}

export const RadioItems: Story = {
  args: {
    trigger: <Button variant="secondary">Alignment</Button>,
    children: null,
  },
  render: () => {
    const [alignment, setAlignment] = useState<Alignment>("Pinapple");
    return (
      <Menu trigger={<Button variant="secondary">{alignment}</Button>}>
        <AlignmentItems value={alignment} onChange={setAlignment} />
      </Menu>
    );
  },
};

export const WithSubmenu: Story = {
  args: {
    trigger: <Button variant="secondary">Open menu</Button>,
    children: null,
  },
  render: () => {
    const [selected, setSelected] = useState("123");
    return (
      <Menu trigger={<Button variant="secondary">{selected}</Button>}>
        <MenuItemWithSubmenu
          submenu={
            <>
              <MenuItem onClick={() => setSelected("EUR €")}>EUR €</MenuItem>
              <MenuItem onClick={() => setSelected("USD $")}>USD $</MenuItem>
              <MenuItem onClick={() => setSelected("GBP £")}>GBP £</MenuItem>
            </>
          }
        >
          Currency
        </MenuItemWithSubmenu>
        <MenuItemWithSubmenu
          submenu={
            <>
              <MenuItem onClick={() => setSelected("Short date")}>
                Short date
              </MenuItem>
              <MenuItem onClick={() => setSelected("Long date")}>
                Long date
              </MenuItem>
            </>
          }
        >
          Date
        </MenuItemWithSubmenu>
      </Menu>
    );
  },
};

export const ContextMenu: Story = {
  args: {
    open: false,
    onClose: () => {},
    anchorPosition: { x: 0, y: 0 },
    children: null,
  },
  parameters: { layout: "centered" },
  render: () => {
    const [open, setOpen] = useState(false);
    const [position, setPosition] = useState({ x: 0, y: 0 });

    return (
      <>
        <div
          role="none"
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            width: 200,
            height: 120,
            border: "1px dashed var(--palette-grey-400)",
            borderRadius: 8,
            color: "var(--palette-grey-500)",
            fontSize: "var(--typography-font-size)",
            userSelect: "none",
            fontFamily: "var(--typography-font-family)",
          }}
          onContextMenu={(e) => {
            e.preventDefault();
            setPosition({ x: e.clientX, y: e.clientY });
            setOpen(true);
          }}
        >
          Right-click here
        </div>
        <Menu
          open={open}
          onClose={() => setOpen(false)}
          anchorPosition={position}
        >
          <MenuItems />
        </Menu>
      </>
    );
  },
};
