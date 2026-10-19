import type { Meta, StoryObj } from "@storybook/react";
import { standardColors } from "../components/ColorPicker/util";
import {
  DEFAULT_OPTIONS,
  type InvertColorOptions,
  invertColor,
  uninvertColor,
} from "./invertColor";
import { darkThemeVariables, defaultThemeVariables } from "./theme";

// A playground for the dark-mode color inversion (invertColor.ts): every
// palette is shown as stored in the workbook (light mode) and as it is shown
// in dark mode, with the color recovered by `uninvertColor` next to it.
// The options are the knobs of `DEFAULT_OPTIONS`.

const PALETTES: { name: string; colors: string[] }[] = [
  { name: "Standard colors", colors: standardColors },
  {
    name: "Greys",
    colors: [
      "#000000",
      "#262626",
      "#404040",
      "#595959",
      "#808080",
      "#A6A6A6",
      "#BFBFBF",
      "#D9D9D9",
      "#F2F2F2",
      "#FFFFFF",
    ],
  },
  {
    name: "Pastel fills",
    colors: [
      "#FFF2CC",
      "#FCE4D6",
      "#FFC7CE",
      "#E2EFDA",
      "#C6EFCE",
      "#DDEBF7",
      "#D9E1F2",
      "#E4DFEC",
      "#EDEDED",
      "#FFEB9C",
    ],
  },
  {
    name: "Strong fills and fonts",
    colors: [
      "#9C0006",
      "#C00000",
      "#ED7D31",
      "#FFC000",
      "#70AD47",
      "#006100",
      "#5B9BD5",
      "#4472C4",
      "#002060",
      "#7030A0",
    ],
  },
  {
    name: "Conditional formatting",
    colors: [
      "#8CB354",
      "#F8CD3D",
      "#EC5753",
      "#6AAB42",
      "#E05C53",
      "#3458B7",
      "#523E93",
      "#FF2D55",
      "#FFD700",
      "#333333",
    ],
  },
];

type Props = InvertColorOptions;

function Swatch({
  hex,
  background,
  textColor,
  label,
}: {
  hex: string;
  background: string;
  textColor: string;
  label?: string;
}) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 4,
        background,
        color: textColor,
        padding: 8,
        width: 72,
      }}
    >
      <div
        style={{
          width: 40,
          height: 40,
          borderRadius: 6,
          background: hex,
          boxShadow: "0 0 0 1px rgba(128, 128, 128, 0.4)",
        }}
      />
      <code style={{ fontSize: 10 }}>{hex}</code>
      {label ? <span style={{ fontSize: 10 }}>{label}</span> : null}
    </div>
  );
}

function InvertColorPlayground(options: Props) {
  const lightBackground = defaultThemeVariables["--palette-common-white"];
  const lightText = defaultThemeVariables["--palette-sheet-default-text-color"];
  const darkBackground = darkThemeVariables["--palette-common-white"];
  const darkText = darkThemeVariables["--palette-sheet-default-text-color"];
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 24,
        fontFamily: "Inter, sans-serif",
        fontSize: 12,
      }}
    >
      {PALETTES.map(({ name, colors }) => (
        <div key={name}>
          <div style={{ marginBottom: 8, fontWeight: 600 }}>{name}</div>
          <div style={{ display: "flex", gap: 16 }}>
            <div style={{ display: "flex", background: lightBackground }}>
              {colors.map((hex) => (
                <Swatch
                  key={hex}
                  hex={hex}
                  background={lightBackground}
                  textColor={lightText}
                />
              ))}
            </div>
            <div style={{ display: "flex", background: darkBackground }}>
              {colors.map((hex) => {
                const inverted = invertColor(hex, options);
                const back = uninvertColor(inverted, options);
                return (
                  <Swatch
                    key={hex}
                    hex={inverted}
                    background={darkBackground}
                    textColor={darkText}
                    label={back === hex.toUpperCase() ? "↺ exact" : `↺ ${back}`}
                  />
                );
              })}
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}

const meta = {
  title: "Theme/Color inversion",
  component: InvertColorPlayground,
  parameters: {
    layout: "padded",
  },
  args: { ...DEFAULT_OPTIONS },
  argTypes: {
    darkBackgroundL: {
      control: { type: "range", min: 0, max: 1, step: 0.005 },
      description: "OKLab lightness pure white is mapped to",
    },
    darkForegroundL: {
      control: { type: "range", min: 0, max: 1, step: 0.005 },
      description: "OKLab lightness pure black is mapped to",
    },
    lightnessGamma: {
      control: { type: "range", min: 0.25, max: 3, step: 0.05 },
      description:
        "Curve applied to the distance from white (<1 lifts light colors)",
    },
    chromaGamma: {
      control: { type: "range", min: 0.25, max: 3, step: 0.05 },
      description: "Curve applied to the relative chroma (>1 mutes colors)",
    },
  },
} satisfies Meta<typeof InvertColorPlayground>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Playground: Story = {};
