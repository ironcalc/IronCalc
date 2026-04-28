import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import ClassicRule from "./ClassicRule";
import ColorScaleRule, {
  type ColorScaleRuleData,
  steppedGradient,
} from "./ColorScaleRule";
import DataBarsRule, {
  DataBarMiniChart,
  type DataBarsRuleData,
} from "./DataBarsRule";
import type { FormatStyle } from "./FormatStylePicker";
import IconSetsRule, {
  type IconPreviewInfo,
  type IconSetsRuleData,
} from "./IconSetsRule";
import "./edit-rule.css";

export interface RuleData {
  applyTo: string;
  ruleType: string;
  ruleOperator: string;
  ruleValue: string;
  ruleValue2: string;
  formatStyle: FormatStyle;
  colorScale?: ColorScaleRuleData;
  dataBars?: DataBarsRuleData;
  iconSets?: IconSetsRuleData;
}

interface EditRuleProps {
  onSave: (data: RuleData) => void;
  onCancel: () => void;
  initialValues?: RuleData;
  getSelectedArea: () => string;
  resolveValue?: (val: string) => string;
}

const DEFAULT_FORMAT_STYLE: FormatStyle = {
  bold: false,
  italic: false,
  underline: false,
  strike: false,
  fontColor: "#C0392B",
  fillColor: "#FADBD8",
};

type TabId = "classic" | "color_scale" | "data_bars" | "icon_sets";

const EditRule = ({
  onSave,
  onCancel,
  initialValues,
  getSelectedArea,
  resolveValue,
}: EditRuleProps) => {
  const { t } = useTranslation();
  const initialTab: TabId =
    initialValues?.ruleType === "color_scale"
      ? "color_scale"
      : initialValues?.ruleType === "data_bars"
        ? "data_bars"
        : initialValues?.ruleType === "icon_sets"
          ? "icon_sets"
          : "classic";
  const [activeTab, setActiveTab] = useState<TabId>(initialTab);
  const [applyTo, setApplyTo] = useState(initialValues?.applyTo ?? "");
  const [formatStyle, setFormatStyle] = useState<FormatStyle>(
    initialValues?.formatStyle ?? DEFAULT_FORMAT_STYLE,
  );
  const [classicDescription, setClassicDescription] = useState("");
  const [dbPreview, setDbPreview] = useState<DataBarsRuleData>(
    initialValues?.dataBars ?? {
      color: "#6AAB42",
      gradient: false,
      positiveColor: "#6AAB42",
      negativeColor: "#E05C53",
      showBorders: false,
      hideCellContent: false,
      roundCorners: false,
    },
  );
  const [isPreview, setIsPreview] = useState<IconPreviewInfo | null>(null);
  const [csPreviewColors, setCsPreviewColors] = useState<
    [string, string, string]
  >(
    initialValues?.colorScale
      ? [
          initialValues.colorScale.minimum.color,
          initialValues.colorScale.midpoint.color,
          initialValues.colorScale.maximum.color,
        ]
      : ["#8CB354", "#F8CD3D", "#EC5753"],
  );

  const headerPreviewStyle: React.CSSProperties = {
    fontWeight: formatStyle.bold ? "bold" : "normal",
    fontStyle: formatStyle.italic ? "italic" : "normal",
    textDecoration:
      [
        formatStyle.underline ? "underline" : "",
        formatStyle.strike ? "line-through" : "",
      ]
        .filter(Boolean)
        .join(" ") || "none",
    color: formatStyle.fontColor || "#000000",
    backgroundColor: formatStyle.fillColor || "transparent",
  };

  const tabs: { id: TabId; label: string }[] = [
    { id: "classic", label: t("conditional_formatting.tab_classic") },
    { id: "color_scale", label: t("conditional_formatting.tab_color_scale") },
    { id: "data_bars", label: t("conditional_formatting.tab_data_bars") },
    { id: "icon_sets", label: t("conditional_formatting.tab_icon_sets") },
  ];

  return (
    <div className="ic-edit-rule-container">
      <div className="ic-edit-rule-header-box">
        <div
          className="ic-edit-rule-header-preview"
          style={
            activeTab === "color_scale"
              ? { background: steppedGradient(csPreviewColors) }
              : activeTab === "data_bars"
                ? { position: "relative" }
                : activeTab === "icon_sets"
                  ? { background: "var(--palette-common-white)" }
                  : headerPreviewStyle
          }
        >
          {activeTab === "data_bars" ? (
            <DataBarMiniChart
              color={dbPreview.color}
              gradient={dbPreview.gradient}
            />
          ) : activeTab === "icon_sets" && isPreview ? (
            <isPreview.Icon
              size={18}
              color={isPreview.color}
              fill={isPreview.filled ? isPreview.color : "none"}
            />
          ) : (
            activeTab !== "color_scale" && "Aa"
          )}
        </div>
        <span className="ic-edit-rule-header-box-text">
          {activeTab === "classic" && classicDescription
            ? classicDescription
            : tabs.find((tab) => tab.id === activeTab)?.label}
        </span>
      </div>
      <div className="ic-edit-rule-tabs">
        {tabs.map(({ id, label }) => (
          <Button
            key={id}
            variant="ghost"
            size="sm"
            className={`ic-edit-rule-tab${activeTab === id ? " ic-edit-rule-tab--active" : ""}`}
            onClick={() => setActiveTab(id)}
          >
            {label}
          </Button>
        ))}
      </div>
      {activeTab === "classic" && (
        <ClassicRule
          onSave={onSave}
          onCancel={onCancel}
          initialValues={initialValues}
          getSelectedArea={getSelectedArea}
          applyTo={applyTo}
          onApplyToChange={setApplyTo}
          formatStyle={formatStyle}
          onFormatStyleChange={setFormatStyle}
          onDescriptionChange={setClassicDescription}
          resolveValue={resolveValue}
        />
      )}
      {activeTab === "color_scale" && (
        <ColorScaleRule
          onSave={(colorScale) =>
            onSave({
              applyTo,
              ruleType: "color_scale",
              ruleOperator: "",
              ruleValue: "",
              ruleValue2: "",
              formatStyle: DEFAULT_FORMAT_STYLE,
              colorScale,
            })
          }
          onCancel={onCancel}
          applyTo={applyTo}
          onApplyToChange={setApplyTo}
          initialValues={initialValues?.colorScale}
          onPreviewChange={setCsPreviewColors}
          getSelectedArea={getSelectedArea}
        />
      )}
      {activeTab === "data_bars" && (
        <DataBarsRule
          onSave={(dataBars) =>
            onSave({
              applyTo,
              ruleType: "data_bars",
              ruleOperator: "",
              ruleValue: "",
              ruleValue2: "",
              formatStyle: DEFAULT_FORMAT_STYLE,
              dataBars,
            })
          }
          onCancel={onCancel}
          applyTo={applyTo}
          onApplyToChange={setApplyTo}
          getSelectedArea={getSelectedArea}
          initialValues={initialValues?.dataBars}
          onPreviewChange={setDbPreview}
        />
      )}
      {activeTab === "icon_sets" && (
        <IconSetsRule
          onSave={(iconSets) =>
            onSave({
              applyTo,
              ruleType: "icon_sets",
              ruleOperator: "",
              ruleValue: "",
              ruleValue2: "",
              formatStyle: DEFAULT_FORMAT_STYLE,
              iconSets,
            })
          }
          applyTo={applyTo}
          onApplyToChange={setApplyTo}
          getSelectedArea={getSelectedArea}
          onCancel={onCancel}
          onPreviewChange={setIsPreview}
        />
      )}
    </div>
  );
};

export default EditRule;
