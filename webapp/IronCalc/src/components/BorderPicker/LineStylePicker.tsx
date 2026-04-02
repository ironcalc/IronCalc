import { BorderStyle } from "@ironcalc/wasm";
import "./line-style-picker.css";

const STYLE_OPTIONS = [
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

type LineStylePickerProps = {
  value: BorderStyle;
  onSelect: (style: BorderStyle) => void;
};

export default function LineStylePicker({
  value,
  onSelect,
}: LineStylePickerProps) {
  return (
    <div className="ic-line-style-picker">
      {STYLE_OPTIONS.map((option) => (
        <button
          key={option.value}
          type="button"
          className={
            value === option.value
              ? "ic-border-picker__button selected"
              : "ic-border-picker__button"
          }
          onClick={() => onSelect(option.value)}
        >
          <span className={option.previewClassName} />
        </button>
      ))}
    </div>
  );
}
