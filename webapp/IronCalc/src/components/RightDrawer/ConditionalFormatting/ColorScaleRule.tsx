import { Check, SquareMousePointer } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import { IconButton } from "../../Button/IconButton";
import ColorPicker from "../../ColorPicker/ColorPicker";
import { Input } from "../../Input/Input";
import { Select } from "../../Select/Select";
import { Tooltip } from "../../Tooltip/Tooltip";

const TYPE_OPTIONS = [
  "auto",
  "number",
  "percent",
  "formula",
  "percentile",
] as const;
type StopType = (typeof TYPE_OPTIONS)[number];

export interface ColorScaleStop {
  type: StopType;
  value: string;
  color: string;
}

export interface ColorScaleRuleData {
  minimum: ColorScaleStop;
  midpoint: ColorScaleStop;
  maximum: ColorScaleStop;
}

interface ColorScaleRuleProps {
  onSave: (data: ColorScaleRuleData) => void;
  onCancel: () => void;
  applyTo: string;
  initialValues?: ColorScaleRuleData;
  onPreviewChange?: (colors: [string, string, string]) => void;
  getSelectedArea: () => string;
}

interface ColorScalePreset {
  id: string;
  colors: string[];
}

const PRESETS: ColorScalePreset[] = [
  // Row 1 — 3-stop diverging
  { id: "r-y-g", colors: ["#EC5753", "#F8CD3D", "#8CB354"] },
  { id: "g-y-r", colors: ["#8CB354", "#F8CD3D", "#EC5753"] },
  { id: "r-w-g", colors: ["#EC5753", "#FFFFFF", "#8CB354"] },
  { id: "g-w-r", colors: ["#8CB354", "#FFFFFF", "#EC5753"] },
  { id: "r-w-b", colors: ["#EC5753", "#FFFFFF", "#3458B7"] },
  { id: "b-w-r", colors: ["#3458B7", "#FFFFFF", "#EC5753"] },
  { id: "g-w-p", colors: ["#8CB354", "#FFFFFF", "#523E93"] },
  { id: "p-w-g", colors: ["#523E93", "#FFFFFF", "#8CB354"] },
  // Row 2 — 2-stop sequential + others
  { id: "g-lg", colors: ["#8CB354", "#E8F0DD"] },
  { id: "lg-g", colors: ["#E8F0DD", "#8CB354"] },
  { id: "r-lr", colors: ["#EC5753", "#FBDDDD"] },
  { id: "lr-r", colors: ["#FBDDDD", "#EC5753"] },
  { id: "r-y-b", colors: ["#EC5753", "#F2994A", "#F8CD3D", "#8CB354", "#3458B7"] },
  { id: "b-y-r", colors: ["#3458B7", "#8CB354", "#F8CD3D", "#F2994A", "#EC5753"] },
  { id: "dp-m-r", colors: ["#35193E", "#AC0070", "#E13342", "#F37651", "#F6B48F"] },
  { id: "r-m-dp", colors: ["#F6B48F", "#F37651", "#E13342", "#AC0070", "#35193E"] },
];

const STEPS = 5;

function hexToRgb(hex: string): [number, number, number] {
  return [
    parseInt(hex.slice(1, 3), 16),
    parseInt(hex.slice(3, 5), 16),
    parseInt(hex.slice(5, 7), 16),
  ];
}

function rgbToHex(r: number, g: number, b: number): string {
  return `#${[r, g, b].map((v) => Math.round(v).toString(16).padStart(2, "0")).join("")}`;
}

function interpolateColor(c1: string, c2: string, t: number): string {
  const [r1, g1, b1] = hexToRgb(c1);
  const [r2, g2, b2] = hexToRgb(c2);
  return rgbToHex(r1 + (r2 - r1) * t, g1 + (g2 - g1) * t, b1 + (b2 - b1) * t);
}

function sampleColorAt(colors: string[], t: number): string {
  const pos = t * (colors.length - 1);
  const lo = Math.floor(pos);
  const hi = Math.min(lo + 1, colors.length - 1);
  return interpolateColor(colors[lo], colors[hi], pos - lo);
}

export function steppedGradient(colors: string[]): string {
  const bands: string[] = Array.from({ length: STEPS }, (_, i) => {
    const t = i / (STEPS - 1);
    const pos = t * (colors.length - 1);
    const lo = Math.floor(pos);
    const hi = Math.min(lo + 1, colors.length - 1);
    return interpolateColor(colors[lo], colors[hi], pos - lo);
  });

  const pct = 100 / STEPS;
  const stops = bands
    .map((color, i) => `${color} ${i * pct}%, ${color} ${(i + 1) * pct}%`)
    .join(", ");

  return `linear-gradient(to bottom, ${stops})`;
}

interface StopRowProps {
  label: string;
  stop: ColorScaleStop;
  onChange: (stop: ColorScaleStop) => void;
  getSelectedArea: () => string;
}

const StopRow = ({ label, stop, onChange, getSelectedArea }: StopRowProps) => {
  const { t } = useTranslation();
  const [colorOpen, setColorOpen] = useState(false);
  const colorRef = useRef<HTMLButtonElement>(null);

  const typeOptions = TYPE_OPTIONS.map((v) => ({
    value: v,
    label: t(`conditional_formatting.color_scale_type_${v}`),
  }));

  return (
    <div className="ic-edit-rule-field-wrapper">
      <span className="ic-edit-rule-label">{label}</span>
      <div className="ic-cs-settings-row">
        <Select
          value={stop.type}
          options={typeOptions}
          onChange={(v) => onChange({ ...stop, type: v as StopType })}
          hideCheck
        />
        {stop.type !== "auto" && (
          <Input
            type="text"
            value={stop.value}
            onChange={(e) => onChange({ ...stop, value: e.target.value })}
            endAdornment={
              <Tooltip title={t("conditional_formatting.use_selection")}>
                <IconButton
                  size="sm"
                  variant="secondary"
                  icon={<SquareMousePointer />}
                  aria-label={t("conditional_formatting.use_selection")}
                  onClick={() => onChange({ ...stop, value: getSelectedArea() })}
                  className="ic-edit-rule-range-button"
                />
              </Tooltip>
            }
          />
        )}
        <div className="ic-input-control md ic-cs-color-swatch-wrapper">
          <button
            ref={colorRef}
            type="button"
            className="ic-cs-color-swatch"
            style={{ backgroundColor: stop.color }}
            onClick={() => setColorOpen(true)}
            aria-label={label}
          />
        </div>
        <ColorPicker
          color={stop.color}
          defaultColor={stop.color}
          title={t("color_picker.default")}
          onChange={(c) => { onChange({ ...stop, color: c }); setColorOpen(false); }}
          onClose={() => setColorOpen(false)}
          anchorEl={colorRef}
          open={colorOpen}
        />
      </div>
    </div>
  );
};

const ColorScaleRule = ({ onSave, onCancel, applyTo, initialValues, onPreviewChange, getSelectedArea }: ColorScaleRuleProps) => {
  const { t } = useTranslation();
  const [selected, setSelected] = useState<string>(PRESETS[0].id);
  const [minimum, setMinimum] = useState<ColorScaleStop>(initialValues?.minimum ?? { type: "auto", value: "", color: sampleColorAt(PRESETS[0].colors, 0) });
  const [midpoint, setMidpoint] = useState<ColorScaleStop>(initialValues?.midpoint ?? { type: "auto", value: "", color: sampleColorAt(PRESETS[0].colors, 0.5) });
  const [maximum, setMaximum] = useState<ColorScaleStop>(initialValues?.maximum ?? { type: "auto", value: "", color: sampleColorAt(PRESETS[0].colors, 1) });

  const applyPreset = (colors: string[]) => {
    setMinimum((s) => ({ ...s, color: sampleColorAt(colors, 0) }));
    setMidpoint((s) => ({ ...s, color: sampleColorAt(colors, 0.5) }));
    setMaximum((s) => ({ ...s, color: sampleColorAt(colors, 1) }));
  };

  useEffect(() => {
    onPreviewChange?.([minimum.color, midpoint.color, maximum.color]);
  }, [minimum.color, midpoint.color, maximum.color, onPreviewChange]);

  const isStopValid = (stop: ColorScaleStop) =>
    stop.type === "auto" || stop.value.trim() !== "";

  const canSave =
    applyTo.trim() !== "" &&
    isStopValid(minimum) &&
    isStopValid(midpoint) &&
    isStopValid(maximum);

  return (
    <>
      <div className="ic-edit-rule-content">
        <div className="ic-edit-rule-section">
          <div className="ic-edit-rule-section-title">
            {t("conditional_formatting.tab_color_scale")}
          </div>
          <div className="ic-fsp-presets-section">
            <div className="ic-edit-rule-label">
              {t("conditional_formatting.default_styles")}
            </div>
            <div className="ic-fsp-presets">
              {PRESETS.map((preset) => (
                <button
                  key={preset.id}
                  type="button"
                  aria-pressed={selected === preset.id}
                  className="ic-fsp-preset"
                  style={{ background: steppedGradient(preset.colors) }}
                  onClick={() => { setSelected(preset.id); applyPreset(preset.colors); }}
                />
              ))}
            </div>
          </div>
        </div>
        <div className="ic-edit-rule-section">
          <div className="ic-edit-rule-section-title">
            {t("conditional_formatting.color_scale_settings")}
          </div>
          <StopRow
            label={t("conditional_formatting.color_scale_minimum")}
            stop={minimum}
            onChange={setMinimum}
            getSelectedArea={getSelectedArea}
          />
          <StopRow
            label={t("conditional_formatting.color_scale_midpoint")}
            stop={midpoint}
            onChange={setMidpoint}
            getSelectedArea={getSelectedArea}
          />
          <StopRow
            label={t("conditional_formatting.color_scale_maximum")}
            stop={maximum}
            onChange={setMaximum}
            getSelectedArea={getSelectedArea}
          />
        </div>
      </div>
      <div className="ic-edit-rule-footer">
        <Button variant="secondary" onClick={onCancel}>
          {t("conditional_formatting.cancel")}
        </Button>
        <Button
          startIcon={<Check />}
          disabled={!canSave}
          onClick={() => onSave({ minimum, midpoint, maximum })}
        >
          {t("conditional_formatting.apply")}
        </Button>
      </div>
    </>
  );
};

export default ColorScaleRule;
