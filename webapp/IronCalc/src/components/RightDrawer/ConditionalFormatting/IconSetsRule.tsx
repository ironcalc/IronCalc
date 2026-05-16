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
  CircleAlert,
  Diamond,
  Flag,
  Minus,
  SquareMousePointer,
  Star,
  X,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import { IconButton } from "../../Button/IconButton";
import ColorPicker from "../../ColorPicker/ColorPicker";
import IconPicker, { iconSpecFor } from "../../IconPicker/IconPicker";
import { Input } from "../../Input/Input";
import { Select } from "../../Select/Select";
import { Tooltip } from "../../Tooltip/Tooltip";

interface IconSetIcon {
  Icon: LucideIcon;
  color: string;
  filled?: boolean;
  backendName: string;
}

interface IconSetPreset {
  id: string;
  icons: IconSetIcon[];
}

const DIRECTIONAL: IconSetPreset[] = [
  {
    id: "dir-3-arrows-color",
    icons: [
      { Icon: ArrowUp, color: "#8CB354", backendName: "ArrowUp" },
      { Icon: ArrowRight, color: "#F8CD3D", backendName: "ArrowRight" },
      { Icon: ArrowDown, color: "#EC5753", backendName: "ArrowDown" },
    ],
  },
  {
    id: "dir-3-chevrons-color",
    icons: [
      { Icon: ChevronUp, color: "#8CB354", backendName: "TriangleUp" },
      { Icon: Minus, color: "#F8CD3D", backendName: "FlatRectangle" },
      { Icon: ChevronDown, color: "#EC5753", backendName: "TriangleDown" },
    ],
  },
  {
    id: "dir-4-arrows-color",
    icons: [
      { Icon: ArrowUp, color: "#8CB354", backendName: "ArrowUp" },
      { Icon: ArrowUpRight, color: "#F8CD3D", backendName: "ArrowAngleUp" },
      { Icon: ArrowDownRight, color: "#F8CD3D", backendName: "ArrowAngleDown" },
      { Icon: ArrowDown, color: "#EC5753", backendName: "ArrowDown" },
    ],
  },
  {
    id: "dir-5-arrows-color",
    icons: [
      { Icon: ArrowUp, color: "#8CB354", backendName: "ArrowUp" },
      { Icon: ArrowUpRight, color: "#B4D67E", backendName: "ArrowAngleUp" },
      { Icon: ArrowRight, color: "#F8CD3D", backendName: "ArrowRight" },
      { Icon: ArrowDownRight, color: "#FC9F6E", backendName: "ArrowAngleDown" },
      { Icon: ArrowDown, color: "#EC5753", backendName: "ArrowDown" },
    ],
  },
];

const SHAPES: IconSetPreset[] = [
  {
    id: "shapes-3-circles-color",
    icons: [
      { Icon: Circle, color: "#8CB354", filled: true, backendName: "Circle" },
      { Icon: Circle, color: "#F8CD3D", filled: true, backendName: "Circle" },
      { Icon: Circle, color: "#EC5753", filled: true, backendName: "Circle" },
    ],
  },
  {
    id: "shapes-4-circles-color",
    icons: [
      { Icon: Circle, color: "#333333", filled: true, backendName: "Circle" },
      { Icon: Circle, color: "#404040", filled: true, backendName: "Circle" },
      { Icon: Circle, color: "#808080", filled: true, backendName: "Circle" },
      { Icon: Circle, color: "#D9D9D9", filled: true, backendName: "Circle" },
    ],
  },
  {
    id: "shapes-3-multiple",
    icons: [
      { Icon: Diamond, color: "#8CB354", filled: true, backendName: "Rhombus" },
      { Icon: Diamond, color: "#F8CD3D", filled: true, backendName: "Rhombus" },
      { Icon: Diamond, color: "#EC5753", filled: true, backendName: "Rhombus" },
    ],
  },
  {
    id: "shapes-4-circles",
    icons: [
      { Icon: Circle, color: "#333333", filled: true, backendName: "Circle" },
      { Icon: Circle, color: "#757575", filled: true, backendName: "Circle" },
      { Icon: Circle, color: "#EC5753", filled: true, backendName: "Circle" },
      { Icon: Circle, color: "#FF888A", filled: true, backendName: "Circle" },
    ],
  },
];

const INDICATORS: IconSetPreset[] = [
  {
    id: "ind-3-checkx",
    icons: [
      { Icon: Check, color: "#8CB354", backendName: "Check" },
      { Icon: X, color: "#EC5753", backendName: "Cross" },
    ],
  },
  {
    id: "ind-3-check-exclaim-x",
    icons: [
      { Icon: Check, color: "#8CB354", backendName: "Check" },
      { Icon: CircleAlert, color: "#F8CD3D", backendName: "Exclamation" },
      { Icon: X, color: "#EC5753", backendName: "Cross" },
    ],
  },
  {
    id: "ind-4-flags",
    icons: [
      { Icon: Flag, color: "#8CB354", filled: true, backendName: "Flag" },
      { Icon: Flag, color: "#F8CD3D", filled: true, backendName: "Flag" },
      { Icon: Flag, color: "#EC5753", filled: true, backendName: "Flag" },
    ],
  },
];

const DEFAULT_RATING_COLOR = "#FFD700";
const RATINGS: IconSetPreset[] = [
  {
    id: "rating-3-stars",
    icons: [
      { Icon: Star, color: "#FFD700", filled: true, backendName: "Star" },
      { Icon: Star, color: "#FFD700", filled: true, backendName: "Star" },
      { Icon: Star, color: "#FFD700", filled: true, backendName: "Star" },
    ],
  },
  {
    id: "rating-4-stars",
    icons: [
      { Icon: Star, color: "#FFD700", filled: true, backendName: "Star" },
      { Icon: Star, color: "#FFD700", filled: true, backendName: "Star" },
      { Icon: Star, color: "#FFD700", filled: true, backendName: "Star" },
      { Icon: Star, color: "#FFD700", filled: true, backendName: "Star" },
    ],
  },
  {
    id: "rating-5-stars",
    icons: [
      { Icon: Star, color: "#FFD700", filled: true, backendName: "Star" },
      { Icon: Star, color: "#FFD700", filled: true, backendName: "Star" },
      { Icon: Star, color: "#FFD700", filled: true, backendName: "Star" },
      { Icon: Star, color: "#FFD700", filled: true, backendName: "Star" },
      { Icon: Star, color: "#FFD700", filled: true, backendName: "Star" },
    ],
  },
];

export const ALL_PRESETS = [...DIRECTIONAL, ...SHAPES, ...INDICATORS];

const THRESHOLD_TYPE_OPTIONS = [
  "min",
  "max",
  "percent",
  "number",
  "percentile",
  "formula",
] as const;
export type ThresholdType = (typeof THRESHOLD_TYPE_OPTIONS)[number];

interface IconThreshold {
  operator: ">=" | ">";
  value: string;
  type: ThresholdType;
  color: string;
  iconName: string;
}

// Thresholds are ordered HIGH→LOW (index 0 = highest icon bucket, index n-1 = lowest/"else").
// Each threshold's value is the LOWER bound for that icon bucket (like Excel's dialog).
function defaultThresholds(icons: IconSetIcon[]): IconThreshold[] {
  const count = icons.length;
  return icons.map((icon, i) => ({
    operator: ">=" as const,
    value:
      i < count - 1 ? `${Math.round(((count - 1 - i) * 100) / count)}` : "",
    type: "percent",
    color: icon.color,
    iconName: icon.backendName,
  }));
}

function defaultRatingThresholds(
  count: number,
  color: string,
  iconName = "Star",
): IconThreshold[] {
  return Array.from({ length: count }, (_, i) => ({
    operator: ">=" as const,
    value:
      i < count - 1 ? `${Math.round(((count - 1 - i) * 100) / count)}` : "",
    type: "percent",
    color,
    iconName,
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
  rating?: { count: 3 | 4 | 5; icon: string; color: string };
  // Ordered HIGH→LOW: thresholds[0] = highest icon bucket, thresholds[n-1] = lowest/"else".
  // Each threshold's value is the LOWER bound for that bucket (operator ">=" or ">").
  thresholds: {
    operator: ">=" | ">";
    value: string;
    type: ThresholdType;
    color: string;
    iconName: string;
  }[];
  showValue: boolean;
}

interface IconSetsRuleProps {
  onSave: (data: IconSetsRuleData) => void;
  onCancel: () => void;
  applyTo: string;
  onApplyToChange: (val: string) => void;
  getSelectedArea: () => string;
  initialValues?: IconSetsRuleData;
  onPreviewChange?: (icon: IconPreviewInfo) => void;
}

type Mode = "preset" | "rating";

const IconSetsRule = ({
  onSave,
  onCancel,
  applyTo,
  onApplyToChange,
  getSelectedArea,
  initialValues,
  onPreviewChange,
}: IconSetsRuleProps) => {
  const { t } = useTranslation();

  const initialMode: Mode = initialValues?.rating ? "rating" : "preset";
  const [mode, setMode] = useState<Mode>(initialMode);
  const [selected, setSelected] = useState<string>(
    initialValues?.presetId ?? DIRECTIONAL[0].id,
  );
  const [ratingCount, setRatingCount] = useState<3 | 4 | 5>(
    (initialValues?.rating?.count as (3 | 4 | 5) | undefined) ?? 3,
  );
  const [ratingColor, setRatingColor] = useState<string>(
    initialValues?.rating?.color ?? DEFAULT_RATING_COLOR,
  );
  const [ratingIcon, setRatingIcon] = useState<string>(
    initialValues?.rating?.icon ?? "Star",
  );
  const [ratingColorOpen, setRatingColorOpen] = useState(false);
  const ratingColorButtonRef = useRef<HTMLButtonElement | null>(null);

  const [thresholds, setThresholds] = useState<IconThreshold[]>(
    initialValues?.thresholds ??
      (initialMode === "rating"
        ? defaultRatingThresholds(
            initialValues?.rating?.count ?? 3,
            initialValues?.rating?.color ?? DEFAULT_RATING_COLOR,
            initialValues?.rating?.icon ?? "Star",
          )
        : defaultThresholds(DIRECTIONAL[0].icons)),
  );
  const [showValue, setShowValue] = useState<boolean>(
    initialValues?.showValue ?? true,
  );
  const colorButtonRefs = useRef<Map<number, HTMLButtonElement | null>>(
    new Map(),
  );
  const [colorOpenIndex, setColorOpenIndex] = useState<number | null>(null);

  const selectedPreset =
    ALL_PRESETS.find((p) => p.id === selected) ?? DIRECTIONAL[0];

  useEffect(() => {
    if (mode === "preset") {
      onPreviewChange?.(selectedPreset.icons[0]);
    } else {
      const spec = iconSpecFor(ratingIcon);
      onPreviewChange?.({
        Icon: spec.Icon,
        color: ratingColor,
        filled: spec.filled,
      });
    }
  }, [selectedPreset, mode, ratingColor, ratingIcon, onPreviewChange]);

  const typeOptions = THRESHOLD_TYPE_OPTIONS.map((v) => ({
    value: v,
    label: t(`conditional_formatting.color_scale_type_${v}`),
  }));

  const handlePresetSelect = (preset: IconSetPreset) => {
    setMode("preset");
    setSelected(preset.id);
    setThresholds(defaultThresholds(preset.icons));
  };

  const handleRatingCountSelect = (count: 3 | 4 | 5) => {
    setMode("rating");
    setRatingCount(count);
    setThresholds(defaultRatingThresholds(count, ratingColor, ratingIcon));
  };

  const handleRatingColorChange = (color: string) => {
    setRatingColor(color);
    setThresholds((ts) => ts.map((th) => ({ ...th, color })));
    setRatingColorOpen(false);
  };

  const updateThreshold = (i: number, patch: Partial<IconThreshold>) => {
    setThresholds((ts) =>
      ts.map((th, idx) => (idx === i ? { ...th, ...patch } : th)),
    );
  };

  const activeIconCount =
    mode === "rating" ? ratingCount : selectedPreset.icons.length;

  const getLabel = (i: number): string => {
    if (i === activeIconCount - 1) {
      return t("conditional_formatting.icon_sets_else");
    }
    return t("conditional_formatting.icon_sets_when_value_is");
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
  ];

  const handleSave = () => {
    onSave({
      presetId: selected,
      rating:
        mode === "rating"
          ? { count: ratingCount, icon: ratingIcon, color: ratingColor }
          : undefined,
      thresholds,
      showValue,
    });
  };

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
                    selected={mode === "preset" && selected === preset.id}
                    onClick={() => handlePresetSelect(preset)}
                  />
                ))}
              </div>
            </div>
          ))}
        </div>
        <div className="ic-edit-rule-section">
          <div className="ic-edit-rule-section-title">
            {t("conditional_formatting.icon_sets_ratings")}
          </div>
          <div className="ic-fsp-presets-section">
            <div className="ic-edit-rule-label">
              {t("conditional_formatting.icon_sets_ratings")}
            </div>
            <div className="ic-is-presets">
              {RATINGS.map((preset) => (
                <IconSetSwatch
                  key={preset.id}
                  preset={preset}
                  selected={
                    mode === "rating" &&
                    ratingCount === preset.icons.length &&
                    ratingColor === preset.icons[0].color
                  }
                  onClick={() =>
                    handleRatingCountSelect(preset.icons.length as 3 | 4 | 5)
                  }
                />
              ))}
            </div>
          </div>
        </div>
        <div className="ic-edit-rule-section">
          <div className="ic-edit-rule-section-title">
            {t("conditional_formatting.icon_sets_settings")}
          </div>
          {mode === "preset" && (
            <div className="ic-is-threshold-rows">
              {thresholds.map((threshold, i) => {
                const isLast = i === thresholds.length - 1;
                const iconColor = threshold.color;
                let options = typeOptions;
                if (i === 0) {
                  options = typeOptions.filter((o) => o.value !== "max");
                } else if (isLast) {
                  // Although technically it doesn't matter since it is disabled
                  options = typeOptions.filter((o) => o.value !== "min");
                } else {
                  options = typeOptions.filter(
                    (o) => o.value !== "min" && o.value !== "max",
                  );
                }
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
                            threshold.operator === ">=" ? "secondary" : "ghost"
                          }
                          icon={<span className="ic-is-op-text">{"≥"}</span>}
                          aria-label="Greater than or equal"
                          onClick={() => updateThreshold(i, { operator: ">=" })}
                        />
                        <IconButton
                          size="sm"
                          variant={
                            threshold.operator === ">" ? "secondary" : "ghost"
                          }
                          icon={<span className="ic-is-op-text">{">"}</span>}
                          aria-label="Greater than"
                          onClick={() => updateThreshold(i, { operator: ">" })}
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
                          options={options}
                          onChange={(v) =>
                            updateThreshold(i, { type: v as ThresholdType })
                          }
                          hideCheck
                          disabled={isLast}
                        />
                      </div>
                      <div className="ic-input-control md ic-cs-color-swatch-wrapper">
                        <IconPicker
                          value={threshold.iconName}
                          color={iconColor}
                          onChange={(name) =>
                            updateThreshold(i, { iconName: name })
                          }
                        />
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
          )}
          {mode === "rating" && (
            <>
              <div className="ic-is-threshold-rows">
                {thresholds.map((threshold, i) => {
                  if (i === thresholds.length - 1) {
                    // last option is empty
                    return (
                      <div
                        key={`rating${String(i)}`}
                        className="ic-is-threshold-row"
                      ></div>
                    );
                  }
                  let options = typeOptions;
                  if (i === 0) {
                    options = typeOptions.filter((o) => o.value !== "max");
                  } else {
                    options = typeOptions.filter(
                      (o) => o.value !== "min" && o.value !== "max",
                    );
                  }
                  return (
                    <div
                      key={`rating${String(i)}`}
                      className="ic-is-threshold-row"
                    >
                      <div className="ic-is-threshold-label">{getLabel(i)}</div>
                      <div className="ic-is-threshold-controls">
                        <div className="ic-input-control md ic-is-operator-group">
                          <IconButton
                            size="sm"
                            variant={
                              threshold.operator === ">="
                                ? "secondary"
                                : "ghost"
                            }
                            icon={<span className="ic-is-op-text">{"≥"}</span>}
                            aria-label="Greater than or equal"
                            onClick={() =>
                              updateThreshold(i, { operator: ">=" })
                            }
                          />
                          <IconButton
                            size="sm"
                            variant={
                              threshold.operator === ">" ? "secondary" : "ghost"
                            }
                            icon={<span className="ic-is-op-text">{">"}</span>}
                            aria-label="Greater than"
                            onClick={() =>
                              updateThreshold(i, { operator: ">" })
                            }
                          />
                        </div>
                        <div className="ic-is-threshold-value-wrap">
                          <Input
                            type="text"
                            value={threshold.value}
                            onChange={(e) =>
                              updateThreshold(i, { value: e.target.value })
                            }
                          />
                        </div>
                        <div className="ic-is-threshold-type-wrap">
                          <Select
                            value={threshold.type}
                            options={options}
                            onChange={(v) =>
                              updateThreshold(i, { type: v as ThresholdType })
                            }
                            hideCheck
                          />
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
              <div className="ic-fsp-presets-section">
                <div className="ic-edit-rule-settings-row">
                  <span>Icon</span>
                  <div className="ic-input-control md ic-cs-color-swatch-wrapper">
                    <IconPicker
                      value={ratingIcon}
                      color={ratingColor}
                      onChange={setRatingIcon}
                    />
                  </div>
                </div>
                <div className="ic-edit-rule-settings-row">
                  <span>Color</span>
                  <div className="ic-input-control md ic-cs-color-swatch-wrapper">
                    <button
                      ref={ratingColorButtonRef}
                      type="button"
                      className="ic-cs-color-swatch"
                      style={{ backgroundColor: ratingColor }}
                      onClick={() => setRatingColorOpen(true)}
                      aria-label={t("color_picker.title")}
                    />
                  </div>
                  <ColorPicker
                    color={ratingColor}
                    defaultColor={ratingColor}
                    title={t("color_picker.default")}
                    onChange={handleRatingColorChange}
                    onClose={() => setRatingColorOpen(false)}
                    anchorEl={
                      {
                        current: ratingColorButtonRef.current,
                      } as React.RefObject<HTMLButtonElement>
                    }
                    open={ratingColorOpen}
                  />
                </div>
              </div>
            </>
          )}
          <div className="ic-fsp-presets-section">
            <label className="ic-edit-rule-checkbox-row">
              <input
                type="checkbox"
                checked={showValue}
                onChange={(e) => setShowValue(e.target.checked)}
              />
              {t("conditional_formatting.icon_sets_show_value")}
            </label>
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
          onClick={handleSave}
        >
          {t("conditional_formatting.apply")}
        </Button>
      </div>
    </>
  );
};

export default IconSetsRule;
