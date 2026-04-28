import type { LucideIcon } from "lucide-react";
import {
  ArrowDown,
  ArrowDownRight,
  ArrowRight,
  ArrowUp,
  ArrowUpRight,
  Check,
  ChevronDown,
  ChevronUp,
  Circle,
  Diamond,
  Flag,
  Frown,
  Heart,
  Meh,
  Minus,
  Smile,
  Square,
  SquareMousePointer,
  Star,
  ThumbsDown,
  ThumbsUp,
  TrendingDown,
  TrendingUp,
  X,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import { IconButton } from "../../Button/IconButton";
import ColorPicker from "../../ColorPicker/ColorPicker";
import { Input } from "../../Input/Input";
import { Select } from "../../Select/Select";
import { Tooltip } from "../../Tooltip/Tooltip";

interface IconSetIcon {
  Icon: LucideIcon;
  color: string;
  filled?: boolean;
}

interface IconSetPreset {
  id: string;
  icons: IconSetIcon[];
}

const DIRECTIONAL: IconSetPreset[] = [
  {
    id: "dir-3-arrows-color",
    icons: [
      { Icon: ArrowUp, color: "#8CB354" },
      { Icon: ArrowRight, color: "#F8CD3D" },
      { Icon: ArrowDown, color: "#EC5753" },
    ],
  },
  {
    id: "dir-3-chevrons-color",
    icons: [
      { Icon: ChevronUp, color: "#8CB354" },
      { Icon: Minus, color: "#F8CD3D" },
      { Icon: ChevronDown, color: "#EC5753" },
    ],
  },
  {
    id: "dir-4-arrows-color",
    icons: [
      { Icon: ArrowUp, color: "#8CB354" },
      { Icon: ArrowUpRight, color: "#F8CD3D" },
      { Icon: ArrowDownRight, color: "#F8CD3D" },
      { Icon: ArrowDown, color: "#EC5753" },
    ],
  },
  {
    id: "dir-trending",
    icons: [
      { Icon: TrendingUp, color: "#8CB354" },
      { Icon: Minus, color: "#F8CD3D" },
      { Icon: TrendingDown, color: "#EC5753" },
    ],
  },
];

const SHAPES: IconSetPreset[] = [
  {
    id: "shapes-3-circles-color",
    icons: [
      { Icon: Circle, color: "#8CB354", filled: true },
      { Icon: Circle, color: "#F8CD3D", filled: true },
      { Icon: Circle, color: "#EC5753", filled: true },
    ],
  },
  {
    id: "shapes-4-circles-color",
    icons: [
      { Icon: Circle, color: "#333333", filled: true },
      { Icon: Circle, color: "#404040", filled: true },
      { Icon: Circle, color: "#808080", filled: true },
      { Icon: Circle, color: "#D9D9D9", filled: true },
    ],
  },
  {
    id: "shapes-3-multiple",
    icons: [
      { Icon: Diamond, color: "#8CB354", filled: true },
      { Icon: Diamond, color: "#F8CD3D", filled: true },
      { Icon: Diamond, color: "#EC5753", filled: true },
    ],
  },
  {
    id: "shapes-4-circles",
    icons: [
      { Icon: Circle, color: "#333333", filled: true },
      { Icon: Circle, color: "#757575", filled: true },
      { Icon: Circle, color: "#EC5753", filled: true },
      { Icon: Circle, color: "#FF888A", filled: true },
    ],
  },
];

const INDICATORS: IconSetPreset[] = [
  {
    id: "ind-3-checkx",
    icons: [
      { Icon: Check, color: "#8CB354" },
      { Icon: X, color: "#EC5753" },
    ],
  },
  {
    id: "ind-3-thumbs",
    icons: [
      { Icon: ThumbsUp, color: "#8CB354" },
      { Icon: ThumbsDown, color: "#EC5753" },
    ],
  },
  {
    id: "ind-3-emojis",
    icons: [
      { Icon: Smile, color: "#FFC000" },
      { Icon: Meh, color: "#FFC000" },
      { Icon: Frown, color: "#FFC000" },
    ],
  },
  {
    id: "ind-4-flags",
    icons: [
      { Icon: Flag, color: "#8CB354", filled: true },
      { Icon: Flag, color: "#F8CD3D", filled: true },
      { Icon: Flag, color: "#EC5753", filled: true },
    ],
  },
];

const RATINGS: IconSetPreset[] = [
  {
    id: "rat-5-stars",
    icons: [
      { Icon: Star, color: "#F8CD3D", filled: true },
      { Icon: Star, color: "#F8CD3D" },
    ],
  },
  {
    id: "rat-4-stars",
    icons: [
      { Icon: Star, color: "#FFC000", filled: true },
      { Icon: Star, color: "#808080" },
    ],
  },
  {
    id: "rat-5-boxes",
    icons: [
      { Icon: Heart, color: "#EC5753", filled: true },
      { Icon: Heart, color: "#EC5753" },
    ],
  },
  {
    id: "rat-4-boxes",
    icons: [
      { Icon: Square, color: "#404040", filled: true },
      { Icon: Square, color: "#808080" },
    ],
  },
];

export const ALL_PRESETS = [
  ...DIRECTIONAL,
  ...SHAPES,
  ...INDICATORS,
  ...RATINGS,
];

const THRESHOLD_TYPE_OPTIONS = [
  "percent",
  "number",
  "auto",
  "percentile",
  "formula",
] as const;
type ThresholdType = (typeof THRESHOLD_TYPE_OPTIONS)[number];

interface IconThreshold {
  operator: "<" | "<=";
  value: string;
  type: ThresholdType;
  color: string;
}

function defaultThresholds(icons: IconSetIcon[]): IconThreshold[] {
  const count = icons.length;
  return icons.map((icon, i) => ({
    operator: "<=" as const,
    value: i < count - 1 ? String(Math.round(((i + 1) * 100) / count)) : "",
    type: "percent" as ThresholdType,
    color: icon.color,
  }));
}

interface IconSetSwatchProps {
  preset: IconSetPreset;
  selected: boolean;
  onClick: () => void;
}

const IconSetSwatch = ({ preset, selected, onClick }: IconSetSwatchProps) => {
  const size = preset.icons.length >= 5 ? 12 : 14;
  return (
    <button
      type="button"
      onClick={onClick}
      aria-pressed={selected}
      className="ic-fsp-preset ic-is-swatch"
      aria-label={preset.id}
    >
      <div className="ic-is-swatch-icons">
        {preset.icons.map(({ Icon, color, filled }, i) => (
          <Icon
            key={color + String(i)}
            size={size}
            color={color}
            fill={filled ? color : "none"}
          />
        ))}
      </div>
    </button>
  );
};

export interface IconPreviewInfo {
  Icon: LucideIcon;
  color: string;
  filled?: boolean;
}

export interface IconSetsRuleData {
  presetId: string;
  thresholds: {
    operator: "<" | "<=";
    value: string;
    type: ThresholdType;
    color: string;
  }[];
}

interface IconSetsRuleProps {
  onSave: (data: IconSetsRuleData) => void;
  onCancel: () => void;
  applyTo: string;
  onApplyToChange: (val: string) => void;
  getSelectedArea: () => string;
  onPreviewChange?: (icon: IconPreviewInfo) => void;
}

const IconSetsRule = ({
  onSave,
  onCancel,
  applyTo,
  onApplyToChange,
  getSelectedArea,
  onPreviewChange,
}: IconSetsRuleProps) => {
  const { t } = useTranslation();
  const [selected, setSelected] = useState<string>(DIRECTIONAL[0].id);
  const [thresholds, setThresholds] = useState<IconThreshold[]>(
    defaultThresholds(DIRECTIONAL[0].icons),
  );
  const colorButtonRefs = useRef<Map<number, HTMLButtonElement | null>>(
    new Map(),
  );
  const [colorOpenIndex, setColorOpenIndex] = useState<number | null>(null);

  const selectedPreset =
    ALL_PRESETS.find((p) => p.id === selected) ?? DIRECTIONAL[0];

  useEffect(() => {
    onPreviewChange?.(selectedPreset.icons[0]);
  }, [selectedPreset, onPreviewChange]);

  const typeOptions = THRESHOLD_TYPE_OPTIONS.map((v) => ({
    value: v,
    label: t(`conditional_formatting.color_scale_type_${v}`),
  }));

  const handlePresetSelect = (preset: IconSetPreset) => {
    setSelected(preset.id);
    setThresholds(defaultThresholds(preset.icons));
  };

  const updateThreshold = (i: number, patch: Partial<IconThreshold>) => {
    setThresholds((ts) =>
      ts.map((th, idx) => (idx === i ? { ...th, ...patch } : th)),
    );
  };

  const getLabel = (i: number): string => {
    const n = selectedPreset?.icons.length ?? 0;
    if (i === 0) return t("conditional_formatting.icon_sets_when_value_is");
    if (i === n - 1) return t("conditional_formatting.icon_sets_else");
    const prev = thresholds[i - 1];
    const unit =
      prev?.type === "percent" ? "%" : prev?.type === "percentile" ? "th" : "";
    return t("conditional_formatting.icon_sets_when_gte_and", {
      value: prev?.value ?? "",
      unit,
    });
  };

  const groups = [
    {
      key: "directional",
      label: t("conditional_formatting.icon_sets_directional"),
      presets: DIRECTIONAL,
    },
    {
      key: "shapes",
      label: t("conditional_formatting.icon_sets_shapes"),
      presets: SHAPES,
    },
    {
      key: "indicators",
      label: t("conditional_formatting.icon_sets_indicators"),
      presets: INDICATORS,
    },
    {
      key: "ratings",
      label: t("conditional_formatting.icon_sets_ratings"),
      presets: RATINGS,
    },
  ];

  return (
    <>
      <div className="ic-edit-rule-content">
        <div className="ic-edit-rule-section">
          <div className="ic-edit-rule-section-title">
            {t("conditional_formatting.apply_to")}
          </div>
          <div className="ic-edit-rule-field-wrapper">
            <span className="ic-edit-rule-label">
              {t("conditional_formatting.apply_to_range")}
            </span>
            <Input
              type="text"
              placeholder={t("conditional_formatting.apply_to_placeholder")}
              value={applyTo}
              onChange={(e) => onApplyToChange(e.target.value)}
              endAdornment={
                <Tooltip title={t("conditional_formatting.use_selection")}>
                  <IconButton
                    size="sm"
                    variant="secondary"
                    icon={<SquareMousePointer />}
                    aria-label={t("conditional_formatting.use_selection")}
                    onClick={() => onApplyToChange(getSelectedArea())}
                    className="ic-edit-rule-range-button"
                  />
                </Tooltip>
              }
            />
          </div>
        </div>
        <div className="ic-edit-rule-section">
          <div className="ic-edit-rule-section-title">
            {t("conditional_formatting.tab_icon_sets")}
          </div>
          {groups.map((group) => (
            <div key={group.key} className="ic-fsp-presets-section">
              <div className="ic-edit-rule-label">{group.label}</div>
              <div className="ic-is-presets">
                {group.presets.map((preset) => (
                  <IconSetSwatch
                    key={preset.id}
                    preset={preset}
                    selected={selected === preset.id}
                    onClick={() => handlePresetSelect(preset)}
                  />
                ))}
              </div>
            </div>
          ))}
        </div>
        <div className="ic-edit-rule-section">
          <div className="ic-edit-rule-section-title">
            {t("conditional_formatting.icon_sets_settings")}
          </div>
          <div className="ic-is-threshold-rows">
            {selectedPreset.icons.map(({ Icon, filled }, i) => {
              const threshold = thresholds[i];
              if (!threshold) return null;
              const isLast = i === selectedPreset.icons.length - 1;
              return (
                <div
                  key={selectedPreset.id + String(i)}
                  className="ic-is-threshold-row"
                >
                  <div className="ic-is-threshold-label">{getLabel(i)}</div>
                  <div className="ic-is-threshold-controls">
                    <div className="ic-input-control md ic-is-operator-group">
                      <IconButton
                        size="sm"
                        variant={
                          threshold.operator === "<" ? "secondary" : "ghost"
                        }
                        icon={<span className="ic-is-op-text">{"<"}</span>}
                        aria-label="Less than"
                        onClick={() => updateThreshold(i, { operator: "<" })}
                      />
                      <IconButton
                        size="sm"
                        variant={
                          threshold.operator === "<=" ? "secondary" : "ghost"
                        }
                        icon={<span className="ic-is-op-text">{"≤"}</span>}
                        aria-label="Less than or equal"
                        onClick={() => updateThreshold(i, { operator: "<=" })}
                      />
                    </div>
                    <div className="ic-is-threshold-value-wrap">
                      <Input
                        type="text"
                        value={isLast ? "" : threshold.value}
                        onChange={(e) =>
                          updateThreshold(i, { value: e.target.value })
                        }
                        disabled={isLast}
                      />
                    </div>
                    <div className="ic-is-threshold-type-wrap">
                      <Select
                        value={threshold.type}
                        options={typeOptions}
                        onChange={(v) =>
                          updateThreshold(i, { type: v as ThresholdType })
                        }
                        hideCheck
                      />
                    </div>
                    <div className="ic-input-control md ic-cs-color-swatch-wrapper">
                      <button
                        type="button"
                        className="ic-is-threshold-icon-btn"
                        aria-label="Change icon"
                      >
                        <Icon
                          color={threshold.color}
                          fill={filled ? threshold.color : "none"}
                        />
                      </button>
                    </div>
                    <div className="ic-input-control md ic-cs-color-swatch-wrapper">
                      <button
                        ref={(el) => {
                          colorButtonRefs.current.set(i, el);
                        }}
                        type="button"
                        className="ic-cs-color-swatch"
                        style={{ backgroundColor: threshold.color }}
                        onClick={() => setColorOpenIndex(i)}
                        aria-label={t("color_picker.title")}
                      />
                    </div>
                    <ColorPicker
                      color={threshold.color}
                      defaultColor={threshold.color}
                      title={t("color_picker.default")}
                      onChange={(c) => {
                        updateThreshold(i, { color: c });
                        setColorOpenIndex(null);
                      }}
                      onClose={() => setColorOpenIndex(null)}
                      anchorEl={
                        {
                          current: colorButtonRefs.current.get(i) ?? null,
                        } as React.RefObject<HTMLButtonElement>
                      }
                      open={colorOpenIndex === i}
                    />
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </div>
      <div className="ic-edit-rule-footer">
        <Button variant="secondary" onClick={onCancel}>
          {t("conditional_formatting.cancel")}
        </Button>
        <Button
          startIcon={<Check />}
          disabled={!applyTo.trim()}
          onClick={() => onSave({ presetId: selected, thresholds })}
        >
          {t("conditional_formatting.apply")}
        </Button>
      </div>
    </>
  );
};

export default IconSetsRule;
