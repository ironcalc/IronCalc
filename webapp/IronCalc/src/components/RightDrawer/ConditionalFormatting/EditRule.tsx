import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import ClassicRule from "./ClassicRule";
import ColorScaleRule from "./ColorScaleRule";
import DataBarsRule from "./DataBarsRule";
import type { FormatStyle } from "./FormatStylePicker";
import IconSetsRule from "./IconSetsRule";
import "./edit-rule.css";

export interface RuleData {
  applyTo: string;
  ruleType: string;
  ruleOperator: string;
  ruleValue: string;
  ruleValue2: string;
  formatStyle: FormatStyle;
}

interface EditRuleProps {
  onSave: (data: RuleData) => void;
  onCancel: () => void;
  initialValues?: RuleData;
  getSelectedArea: () => string;
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
}: EditRuleProps) => {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<TabId>("classic");
  const [formatStyle, setFormatStyle] = useState<FormatStyle>(
    initialValues?.formatStyle ?? DEFAULT_FORMAT_STYLE,
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
        <div className="ic-edit-rule-header-preview" style={headerPreviewStyle}>
          Aa
        </div>
        <span className="ic-edit-rule-header-box-text">
          {t("conditional_formatting.new_rule")}
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
          formatStyle={formatStyle}
          onFormatStyleChange={setFormatStyle}
        />
      )}
      {activeTab === "color_scale" && <ColorScaleRule onCancel={onCancel} />}
      {activeTab === "data_bars" && <DataBarsRule onCancel={onCancel} />}
      {activeTab === "icon_sets" && <IconSetsRule onCancel={onCancel} />}
    </div>
  );
};

export default EditRule;
