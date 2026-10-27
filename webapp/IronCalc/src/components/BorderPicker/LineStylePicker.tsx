import { BorderStyle } from "@ironcalc/wasm";
import "./line-style-picker.css";

export const STYLE_OPTIONS = [
  { value: BorderStyle.Thin, previewClassName: "thin" },
  { value: BorderStyle.Medium, previewClassName: "medium" },
  { value: BorderStyle.Thick, previewClassName: "thick" },
  { value: BorderStyle.Double, previewClassName: "double" },
  { value: BorderStyle.Dotted, previewClassName: "dotted" },
  { value: BorderStyle.MediumDashed, previewClassName: "medium-dashed" },
  { value: BorderStyle.SlantDashDot, previewClassName: "slant-dash-dot" },
  { value: BorderStyle.MediumDashDot, previewClassName: "medium-dash-dot" },
  {
    value: BorderStyle.MediumDashDotDot,
    previewClassName: "medium-dash-dot-dot",
  },
] as const;
