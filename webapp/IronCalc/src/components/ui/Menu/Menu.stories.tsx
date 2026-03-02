import type { Meta, StoryObj } from "@storybook/react";
import { Check, ClipboardPaste, Copy, Scissors, Trash2 } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Button } from "../Button/Button";
import { Menu, MenuDivider, type MenuProps } from "./Menu";
import { MenuItem } from "./MenuItem";

const meta = {
  title: "UI/Menu",
  component: Menu,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  argTypes: {
    placement: {
      control: "select",
      options: [
        "bottom-start",
        "bottom",
        "bottom-end",
        "top-start",
        "top",
        "top-end",
        "left-start",
        "left",
        "left-end",
        "right-start",
        "right",
        "right-end",
      ],
      description: "Popper placement relative to anchor",
    },
  },
} satisfies Meta<typeof Menu>;

export default meta;

type Story = Omit<StoryObj<typeof meta>, "args"> & {
  args?: Partial<MenuProps>;
};

function FormatMenuLikeContent({
  onSelectFormat,
  selectedFormat = "auto",
}: {
  onSelectFormat: (format: string) => void;
  selectedFormat?: string;
}) {
  const checkOrSpace = (key: string) =>
    selectedFormat === key ? (
      <Check />
    ) : (
      <span style={{ width: 16, height: 16, display: "inline-block" }} />
    );

  return (
    <>
      <MenuItem
        onClick={() => onSelectFormat("auto")}
        selected={selectedFormat === "auto"}
        startAdornment={checkOrSpace("auto")}
      >
        Auto
      </MenuItem>
      <MenuDivider />
      <MenuItem
        onClick={() => onSelectFormat("number")}
        selected={selectedFormat === "number"}
        startAdornment={checkOrSpace("number")}
        endAdornment="1.234,56"
      >
        Number
      </MenuItem>
      <MenuItem
        onClick={() => onSelectFormat("percentage")}
        selected={selectedFormat === "percentage"}
        startAdornment={checkOrSpace("percentage")}
        endAdornment="12.35%"
      >
        Percentage
      </MenuItem>
      <MenuDivider />
      <MenuItem
        onClick={() => onSelectFormat("currency_eur")}
        selected={selectedFormat === "currency_eur"}
        startAdornment={checkOrSpace("currency_eur")}
        endAdornment="€1.234,56"
      >
        Euro (EUR)
      </MenuItem>
      <MenuItem
        onClick={() => onSelectFormat("currency_usd")}
        selected={selectedFormat === "currency_usd"}
        startAdornment={checkOrSpace("currency_usd")}
        endAdornment="$1,234.56"
      >
        Dollar (USD)
      </MenuItem>
      <MenuItem
        onClick={() => onSelectFormat("currency_gbp")}
        selected={selectedFormat === "currency_gbp"}
        startAdornment={checkOrSpace("currency_gbp")}
        endAdornment="£1,234.56"
      >
        British Pound (GBP)
      </MenuItem>
      <MenuDivider />
      <MenuItem
        onClick={() => onSelectFormat("date_short")}
        selected={selectedFormat === "date_short"}
        startAdornment={checkOrSpace("date_short")}
        endAdornment="31/12/2024"
      >
        Short date
      </MenuItem>
      <MenuItem
        onClick={() => onSelectFormat("date_long")}
        selected={selectedFormat === "date_long"}
        startAdornment={checkOrSpace("date_long")}
        endAdornment="31 December 2024"
      >
        Long date
      </MenuItem>
      <MenuDivider />
      <MenuItem
        onClick={() => onSelectFormat("custom")}
        selected={selectedFormat === "custom"}
        startAdornment={checkOrSpace("custom")}
      >
        Custom format…
      </MenuItem>
    </>
  );
}

function MenuWithAnchor({
  placement = "bottom-start",
  offset,
  formatMenuStyle = false,
}: {
  placement?: MenuProps["placement"];
  offset?: [number, number];
  formatMenuStyle?: boolean;
}) {
  const [open, setOpen] = useState(false);
  const [selectedFormat, setSelectedFormat] = useState("auto");
  const anchorRef = useRef<HTMLButtonElement>(null);

  const handleSelectFormat = (format: string) => {
    setSelectedFormat(format);
    setOpen(false);
  };

  return (
    <>
      <Button
        ref={anchorRef}
        variant="outline"
        onClick={() => setOpen((o) => !o)}
      >
        {formatMenuStyle ? "123" : "Open menu"}
      </Button>
      <Menu
        open={open}
        onClose={() => setOpen(false)}
        anchorEl={anchorRef}
        placement={placement}
        offset={offset}
      >
        {formatMenuStyle ? (
          <FormatMenuLikeContent
            onSelectFormat={handleSelectFormat}
            selectedFormat={selectedFormat}
          />
        ) : (
          <>
            <MenuItem onClick={() => setOpen(false)}>Auto</MenuItem>
            <MenuItem onClick={() => setOpen(false)}>Number</MenuItem>
            <MenuItem onClick={() => setOpen(false)}>Percentage</MenuItem>
          </>
        )}
      </Menu>
    </>
  );
}

export const Default: Story = {
  render: () => <MenuWithAnchor />,
};

export const Format: Story = {
  render: (args) => (
    <MenuWithAnchor placement={args.placement} formatMenuStyle />
  ),
  args: {
    placement: "bottom-start",
  },
};

export const WithSelectedItem: Story = {
  render: () => {
    const [open, setOpen] = useState(false);
    const [selected, setSelected] = useState("auto");
    const anchorRef = useRef<HTMLButtonElement>(null);

    return (
      <>
        <Button
          ref={anchorRef}
          variant="outline"
          onClick={() => setOpen((o) => !o)}
        >
          Format: {selected}
        </Button>
        <Menu open={open} onClose={() => setOpen(false)} anchorEl={anchorRef}>
          <MenuItem
            onClick={() => {
              setSelected("auto");
              setOpen(false);
            }}
            selected={selected === "auto"}
            startAdornment={selected === "auto" ? <Check /> : null}
          >
            Auto
          </MenuItem>
          <MenuItem
            onClick={() => {
              setSelected("number");
              setOpen(false);
            }}
            selected={selected === "number"}
            startAdornment={selected === "number" ? <Check /> : null}
          >
            Number
          </MenuItem>
          <MenuItem
            onClick={() => {
              setSelected("percentage");
              setOpen(false);
            }}
            selected={selected === "percentage"}
            startAdornment={selected === "percentage" ? <Check /> : null}
          >
            Percentage
          </MenuItem>
        </Menu>
      </>
    );
  },
};

export const WithDisabledItem: Story = {
  render: () => {
    const [open, setOpen] = useState(false);
    const anchorRef = useRef<HTMLButtonElement>(null);

    return (
      <>
        <Button
          ref={anchorRef}
          variant="outline"
          onClick={() => setOpen((o) => !o)}
        >
          Actions
        </Button>
        <Menu open={open} onClose={() => setOpen(false)} anchorEl={anchorRef}>
          <MenuItem onClick={() => setOpen(false)} startAdornment={<Copy />}>
            Copy
          </MenuItem>
          <MenuItem
            onClick={() => setOpen(false)}
            startAdornment={<Scissors />}
          >
            Cut
          </MenuItem>
          <MenuItem
            onClick={() => setOpen(false)}
            startAdornment={<ClipboardPaste />}
          >
            Paste
          </MenuItem>
          <MenuDivider />
          <MenuItem disabled destructive startAdornment={<Trash2 />}>
            Delete (disabled)
          </MenuItem>
        </Menu>
      </>
    );
  },
};

export const RightAligned: Story = {
  render: () => <MenuWithAnchor placement="bottom-end" />,
};

export const WithSubmenu: Story = {
  render: () => {
    const [open, setOpen] = useState(false);
    const anchorRef = useRef<HTMLButtonElement>(null);

    return (
      <>
        <Button
          ref={anchorRef}
          variant="outline"
          onClick={() => setOpen((o) => !o)}
        >
          Open menu
        </Button>
        <Menu open={open} onClose={() => setOpen(false)} anchorEl={anchorRef}>
          <MenuItem onClick={() => setOpen(false)}>Plain item</MenuItem>
          <MenuDivider />
          <MenuItem
            submenu={
              <>
                <MenuItem onClick={() => setOpen(false)}>Sub item A</MenuItem>
                <MenuItem onClick={() => setOpen(false)}>Sub item B</MenuItem>
                <MenuDivider />
                <MenuItem onClick={() => setOpen(false)}>Sub item C</MenuItem>
              </>
            }
          >
            Hover for submenu
          </MenuItem>
          <MenuItem
            submenu={
              <>
                <MenuItem onClick={() => setOpen(false)}>Number</MenuItem>
                <MenuItem onClick={() => setOpen(false)}>Percentage</MenuItem>
                <MenuItem onClick={() => setOpen(false)}>Currency</MenuItem>
              </>
            }
          >
            Format
          </MenuItem>
        </Menu>
      </>
    );
  },
};

export const TwoMenus: Story = {
  render: () => {
    const [menuAOpen, setMenuAOpen] = useState(false);
    const [menuBOpen, setMenuBOpen] = useState(false);
    const anchorARef = useRef<HTMLButtonElement>(null);
    const anchorBRef = useRef<HTMLButtonElement>(null);

    return (
      <div style={{ display: "flex", gap: 12 }}>
        <Button
          ref={anchorARef}
          variant="outline"
          onClick={() => setMenuAOpen((o) => !o)}
        >
          Menu A
        </Button>
        <Menu
          open={menuAOpen}
          onClose={() => setMenuAOpen(false)}
          anchorEl={anchorARef}
        >
          <MenuItem onClick={() => setMenuAOpen(false)}>Option 1</MenuItem>
          <MenuItem onClick={() => setMenuAOpen(false)}>Option 2</MenuItem>
          <MenuDivider />
          <MenuItem onClick={() => setMenuAOpen(false)}>Option 3</MenuItem>
        </Menu>

        <Button
          ref={anchorBRef}
          variant="outline"
          onClick={() => setMenuBOpen((o) => !o)}
        >
          Menu B
        </Button>
        <Menu
          open={menuBOpen}
          onClose={() => setMenuBOpen(false)}
          anchorEl={anchorBRef}
        >
          <MenuItem onClick={() => setMenuBOpen(false)}>Action X</MenuItem>
          <MenuItem onClick={() => setMenuBOpen(false)}>Action Y</MenuItem>
        </Menu>
      </div>
    );
  },
};

export const ContextMenu: Story = {
  render: () => {
    const [open, setOpen] = useState(false);
    const [anchorPosition, setAnchorPosition] = useState<{
      x: number;
      y: number;
    } | null>(null);
    const anchorRef = useRef<HTMLDivElement>(null);

    const handleClose = () => {
      setOpen(false);
      setAnchorPosition(null);
    };

    // Open menu after the virtual anchor is in the DOM so ref is set
    useEffect(() => {
      if (anchorPosition != null) setOpen(true);
    }, [anchorPosition]);

    return (
      <>
        <button
          type="button"
          aria-label="Right-click to open context menu"
          onContextMenu={(e) => {
            e.preventDefault();
            setAnchorPosition({ x: e.clientX, y: e.clientY });
          }}
          style={{
            padding: 48,
            border: "2px dashed #ccc",
            borderRadius: 8,
            textAlign: "center",
            color: "#888",
            userSelect: "none",
            background: "transparent",
            cursor: "default",
          }}
        >
          Right-click here
        </button>
        {anchorPosition != null && (
          <div
            ref={anchorRef}
            style={{
              position: "fixed",
              left: anchorPosition.x,
              top: anchorPosition.y,
              width: 0,
              height: 0,
            }}
          />
        )}
        <Menu open={open} onClose={handleClose} anchorEl={anchorRef}>
          <MenuItem onClick={handleClose} startAdornment={<Copy />}>
            Copy
          </MenuItem>
          <MenuItem onClick={handleClose} startAdornment={<Scissors />}>
            Cut
          </MenuItem>
          <MenuItem onClick={handleClose} startAdornment={<ClipboardPaste />}>
            Paste
          </MenuItem>
          <MenuDivider />
          <MenuItem
            onClick={handleClose}
            startAdornment={<Trash2 />}
            destructive
          >
            Delete
          </MenuItem>
        </Menu>
      </>
    );
  },
};
