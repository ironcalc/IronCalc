export interface IconPathSpec {
  paths: string[];
  stroke?: boolean;
  fill?: boolean;
  strokeWidth?: number;
  flipY?: boolean;
}

// All icons use a 24×24 viewBox.
// stroke=true → ctx.stroke(path), fill=true (default) → ctx.fill(path)
// Most Lucide icons are stroke-based with strokeWidth=2.
export const ICON_PATH_SPECS: Record<string, IconPathSpec> = {
  ArrowUp: {
    paths: ["m5 12 7-7 7 7", "M12 19V5"],
    stroke: true,
  },
  ArrowRight: {
    paths: ["M5 12h14", "m12 5 7 7-7 7"],
    stroke: true,
  },
  ArrowDown: {
    paths: ["M12 5v14", "m19 12-7 7-7-7"],
    stroke: true,
  },
  // ArrowUpRight
  ArrowAngleUp: {
    paths: ["M7 7h10v10", "M7 17 17 7"],
    stroke: true,
  },
  // ArrowDownRight
  ArrowAngleDown: {
    paths: ["m7 7 10 10", "M17 7v10H7"],
    stroke: true,
  },
  // Triangle (filled) pointing up; TriangleDown uses flipY
  TriangleUp: {
    paths: [
      "M13.73 4a2 2 0 0 0-3.46 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z",
    ],
    fill: true,
  },
  TriangleDown: {
    paths: [
      "M13.73 4a2 2 0 0 0-3.46 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z",
    ],
    fill: true,
    flipY: true,
  },
  // Minus with thick stroke
  FlatRectangle: {
    paths: ["M5 12h14"],
    stroke: true,
    strokeWidth: 3,
  },
  // Circle (filled) — converted from <circle cx="12" cy="12" r="10"/>
  Circle: {
    paths: ["M2 12a10 10 0 1 0 20 0a10 10 0 1 0-20 0"],
    fill: true,
  },
  // Diamond (filled)
  Rhombus: {
    paths: [
      "M2.7 10.3a2.41 2.41 0 0 0 0 3.41l7.59 7.59a2.41 2.41 0 0 0 3.41 0l7.59-7.59a2.41 2.41 0 0 0 0-3.41L13.7 2.71a2.41 2.41 0 0 0-3.41 0Z",
    ],
    fill: true,
  },
  // Flag (OctagonAlert / flag icon)
  Flag: {
    paths: [
      "M4 22V4a1 1 0 0 1 .4-.8A6 6 0 0 1 8 2a8 8 0 0 1 4 1 8 8 0 0 0 4 1 6 6 0 0 0 2.8-.7 1 1 0 0 1 1.2.15 1 1 0 0 1 0 .85V13a1 1 0 0 1-.6.9A6 6 0 0 1 16 14a8 8 0 0 1-4-1 8 8 0 0 0-4-1 6 6 0 0 0-4 1.4",
    ],
    stroke: true,
  },
  // Checkmark
  Check: {
    paths: ["M20 6 9 17l-5-5"],
    stroke: true,
    strokeWidth: 2.5,
  },
  // X mark
  Cross: {
    paths: ["M18 6 6 18", "m6 6 12 12"],
    stroke: true,
    strokeWidth: 2.5,
  },
  // CircleAlert: circle + exclamation
  Exclamation: {
    paths: ["M2 12a10 10 0 1 0 20 0a10 10 0 1 0-20 0", "M12 8v4", "M12 16h.01"],
    stroke: true,
  },
  Signal1: {
    paths: ["M2 20h.01"],
    stroke: true,
    strokeWidth: 3,
  },
  Signal2: {
    paths: ["M2 20h.01", "M7 20v-4"],
    stroke: true,
    strokeWidth: 3,
  },
  Signal3: {
    paths: ["M2 20h.01", "M7 20v-4", "M12 20v-8"],
    stroke: true,
    strokeWidth: 3,
  },
  Signal4: {
    paths: ["M2 20h.01", "M7 20v-4", "M12 20v-8", "M17 20V8"],
    stroke: true,
    strokeWidth: 3,
  },
  Signal5: {
    paths: ["M2 20h.01", "M7 20v-4", "M12 20v-8", "M17 20V8", "M22 4v16"],
    stroke: true,
    strokeWidth: 3,
  },
  Star: {
    paths: [
      "M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z",
    ],
    stroke: false,
  },
};
