import { Check, SquareMousePointer } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import { IconButton } from "../../Button/IconButton";
import ColorPicker from "../../ColorPicker/ColorPicker";
import { Input } from "../../Input/Input";
import { Tooltip } from "../../Tooltip/Tooltip";

export interface DataBarsRuleData {
  color: string;
  gradient: boolean;
  positiveColor: string;
  negativeColor: string;
  showBorders: boolean;
  hideCellContent: boolean;
  roundCorners: boolean;
}

interface DataBarsRuleProps {
  onSave: (data: DataBarsRuleData) => void;
  onCancel: () => void;
  applyTo: string;
  onApplyToChange: (val: string) => void;
  getSelectedArea: () => string;
  initialValues?: DataBarsRuleData;
  onPreviewChange?: (data: DataBarsRuleData) => void;
}

const COLORS = [
  "#8CB354",
  "#EC5753",
  "#3458B7",
  "#F2994A",
  "#F8CD3D",
  "#523E93",
  "#FF2D55",
  "#333333",
];

export const BAR_WIDTHS = [0.6, 0.85, 0.35, 0.55];

interface DataBarSwatchProps {
  color: string;
  gradient: boolean;
  selected: boolean;
  onClick: () => void;
  label: string;
}

export const DataBarMiniChart = ({
  color,
  gradient,
}: {
  color: string;
  gradient: boolean;
}) => {
  const barBackground = gradient
    ? `linear-gradient(to right, ${color}, ${color}1a)`
    : color;
  return (
    <div className="ic-db-mini-chart">
      {BAR_WIDTHS.map((w, i) => (
        <div
          key={w}
          style={{
            flex: 1,
            display: "flex",
            alignItems: "center",
            borderTop: i > 0 ? "1px solid #e0e0e0" : undefined,
          }}
        >
          <div
            style={{
              width: `${w * 100}%`,
              height: "100%",
              background: barBackground,
            }}
          />
        </div>
      ))}
    </div>
  );
};

const DataBarSwatch = ({
  color,
  gradient,
  selected,
  onClick,
  label,
}: DataBarSwatchProps) => {
  const barStyle = {
    background: gradient
      ? `linear-gradient(to right, ${color}, ${color}1a)`
      : color,
  };
  return (
    <button
      type="button"
      onClick={onClick}
      aria-pressed={selected}
      className="ic-fsp-preset ic-db-swatch"
      aria-label={label}
    >
      <div className="ic-db-swatch-inner">
        {BAR_WIDTHS.map((w) => (
          <div key={w} className="ic-db-swatch-row">
            <div
              className="ic-db-swatch-bar"
              style={{ width: `${w * 100}%`, ...barStyle }}
            />
          </div>
        ))}
      </div>
    </button>
  );
};

const DataBarsRule = ({
  onSave,
  onCancel,
  applyTo,
  onApplyToChange,
  getSelectedArea,
  initialValues,
  onPreviewChange,
}: DataBarsRuleProps) => {
  const { t } = useTranslation();
  const [positiveOpen, setPositiveOpen] = useState(false);
  const [negativeOpen, setNegativeOpen] = useState(false);
  const positiveRef = useRef<HTMLButtonElement>(null);
  const negativeRef = useRef<HTMLButtonElement>(null);
  const [selected, setSelected] = useState<DataBarsRuleData>(
    initialValues ?? {
      color: COLORS[0],
      gradient: false,
      positiveColor: "#6AAB42",
      negativeColor: "#E05C53",
      showBorders: false,
      hideCellContent: false,
      roundCorners: false,
    },
  );

  useEffect(() => {
    onPreviewChange?.(selected);
  }, [selected, onPreviewChange]);

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
            {t("conditional_formatting.tab_data_bars")}
          </div>
          <div className="ic-fsp-presets-section">
            <div className="ic-edit-rule-label">
              {t("conditional_formatting.data_bars_solid_fill")}
            </div>
            <div className="ic-fsp-presets">
              {COLORS.map((color) => (
                <DataBarSwatch
                  key={`solid-${color}`}
                  color={color}
                  gradient={false}
                  selected={selected.color === color && !selected.gradient}
                  onClick={() =>
                    setSelected((s) => ({
                      ...s,
                      color,
                      gradient: false,
                      positiveColor: color,
                    }))
                  }
                  label={color}
                />
              ))}
            </div>
          </div>
          <div className="ic-fsp-presets-section">
            <div className="ic-edit-rule-label">
              {t("conditional_formatting.data_bars_gradient_fill")}
            </div>
            <div className="ic-fsp-presets">
              {COLORS.map((color) => (
                <DataBarSwatch
                  key={`gradient-${color}`}
                  color={color}
                  gradient={true}
                  selected={selected.color === color && selected.gradient}
                  onClick={() =>
                    setSelected((s) => ({
                      ...s,
                      color,
                      gradient: true,
                      positiveColor: color,
                    }))
                  }
                  label={color}
                />
              ))}
            </div>
          </div>
        </div>
        <div className="ic-edit-rule-section">
          <div className="ic-edit-rule-section-title">
            {t("conditional_formatting.data_bars_settings")}
          </div>
          <div className="ic-edit-rule-values-row">
            <div className="ic-edit-rule-field-wrapper">
              <span className="ic-edit-rule-label">
                {t("conditional_formatting.data_bars_positive")}
              </span>
              <div className="ic-cs-settings-row">
                <div className="ic-input-control md ic-cs-color-swatch-wrapper">
                  <button
                    ref={positiveRef}
                    type="button"
                    className="ic-cs-color-swatch"
                    style={{ backgroundColor: selected.positiveColor }}
                    onClick={() => setPositiveOpen(true)}
                    aria-label={t("conditional_formatting.data_bars_positive")}
                  />
                </div>
                <ColorPicker
                  color={selected.positiveColor}
                  defaultColor={selected.positiveColor}
                  title={t("color_picker.default")}
                  onChange={(c) => {
                    setSelected((s) => ({ ...s, positiveColor: c }));
                    setPositiveOpen(false);
                  }}
                  onClose={() => setPositiveOpen(false)}
                  anchorEl={positiveRef}
                  open={positiveOpen}
                />
                <Input
                  type="text"
                  value={selected.positiveColor}
                  onChange={(e) =>
                    setSelected((s) => ({
                      ...s,
                      positiveColor: e.target.value,
                    }))
                  }
                />
              </div>
            </div>
            <div className="ic-edit-rule-field-wrapper">
              <span className="ic-edit-rule-label">
                {t("conditional_formatting.data_bars_negative")}
              </span>
              <div className="ic-cs-settings-row">
                <div className="ic-input-control md ic-cs-color-swatch-wrapper">
                  <button
                    ref={negativeRef}
                    type="button"
                    className="ic-cs-color-swatch"
                    style={{ backgroundColor: selected.negativeColor }}
                    onClick={() => setNegativeOpen(true)}
                    aria-label={t("conditional_formatting.data_bars_negative")}
                  />
                </div>
                <ColorPicker
                  color={selected.negativeColor}
                  defaultColor={selected.negativeColor}
                  title={t("color_picker.default")}
                  onChange={(c) => {
                    setSelected((s) => ({ ...s, negativeColor: c }));
                    setNegativeOpen(false);
                  }}
                  onClose={() => setNegativeOpen(false)}
                  anchorEl={negativeRef}
                  open={negativeOpen}
                />
                <Input
                  type="text"
                  value={selected.negativeColor}
                  onChange={(e) =>
                    setSelected((s) => ({
                      ...s,
                      negativeColor: e.target.value,
                    }))
                  }
                />
              </div>
            </div>
          </div>
          <div className="ic-fsp-presets-section">
            <div className="ic-edit-rule-label">
              {t("conditional_formatting.data_bars_preferences")}
            </div>
            <label className="ic-edit-rule-checkbox-row">
              <input
                type="checkbox"
                checked={selected.showBorders}
                onChange={(e) =>
                  setSelected((s) => ({ ...s, showBorders: e.target.checked }))
                }
              />
              {t("conditional_formatting.data_bars_show_borders")}
            </label>
            <label className="ic-edit-rule-checkbox-row">
              <input
                type="checkbox"
                checked={selected.hideCellContent}
                onChange={(e) =>
                  setSelected((s) => ({
                    ...s,
                    hideCellContent: e.target.checked,
                  }))
                }
              />
              {t("conditional_formatting.data_bars_hide_cell_content")}
            </label>
            <label className="ic-edit-rule-checkbox-row">
              <input
                type="checkbox"
                checked={selected.roundCorners}
                onChange={(e) =>
                  setSelected((s) => ({ ...s, roundCorners: e.target.checked }))
                }
              />
              {t("conditional_formatting.data_bars_round_corners")}
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
          onClick={() => onSave(selected)}
        >
          {t("conditional_formatting.apply")}
        </Button>
      </div>
    </>
  );
};

export default DataBarsRule;
