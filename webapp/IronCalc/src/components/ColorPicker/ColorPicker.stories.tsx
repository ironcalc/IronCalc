import type { Color, IronCalcTheme } from "@ironcalc/wasm";
import { getThemeList } from "@ironcalc/wasm";
import type { Meta, StoryObj } from "@storybook/react";
import { useEffect, useRef, useState } from "react";
import { init } from "../../index";
import ColorPicker from "./ColorPicker";

// Names of the builtin themes returned by getThemeList()
const THEME_NAMES = [
  "IronCalc",
  "Office",
  "Greyscale",
  "Cold",
  "Warm",
  "Ocean",
  "Pastel",
  "Dark",
  "Forest",
] as const;

interface ColorPickerStoryProps {
  theme: (typeof THEME_NAMES)[number];
}

function ColorPickerStory({ theme: themeName }: ColorPickerStoryProps) {
  // A theme tuple (accent1), so the selected swatch follows the theme
  const [color, setColor] = useState<Color>([4, 0]);
  // The color grid is computed with wasm (hexWithTintToRgb), so the module
  // must be initialized before rendering the picker.
  const [themes, setThemes] = useState<IronCalcTheme[] | null>(null);
  const anchorRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let cancelled = false;
    init().then(() => {
      if (!cancelled) {
        setThemes(getThemeList());
      }
    });
    return () => {
      cancelled = true;
    };
  }, []);

  if (!themes) {
    return <div>Loading...</div>;
  }

  const theme = themes.find((t) => t.name === themeName) ?? themes[0];

  return (
    <>
      {/* Invisible anchor placed so the picker (~220x360px) lands centered */}
      <div
        ref={anchorRef}
        style={{
          position: "fixed",
          left: "calc(50% - 110px)",
          top: "calc(50% - 184px)",
          width: 0,
          height: 0,
        }}
      />
      <ColorPicker
        color={color}
        defaultColor="#000000"
        title="Default"
        onChange={setColor}
        onClose={() => {}}
        anchorEl={anchorRef}
        open={true}
        theme={theme}
      />
    </>
  );
}

const meta = {
  title: "Components/ColorPicker",
  component: ColorPickerStory,
  parameters: {
    layout: "fullscreen",
  },
  args: {
    theme: "IronCalc",
  },
  argTypes: {
    theme: {
      control: "select",
      options: THEME_NAMES,
      description: "Builtin workbook theme used for the themed colors grid",
    },
  },
} satisfies Meta<typeof ColorPickerStory>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {};
