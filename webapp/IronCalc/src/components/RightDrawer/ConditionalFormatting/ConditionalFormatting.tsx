import type { Model } from "@ironcalc/wasm";
import {
  ArrowLeft,
  PackageOpen,
  PencilLine,
  Plus,
  Search,
  SearchX,
  Trash2,
  X,
} from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import { IconButton } from "../../Button/IconButton";
import { parseRangeInSheet } from "../../Editor/util";
import { Input } from "../../Input/Input";
import { Tooltip } from "../../Tooltip/Tooltip";
import EditRule, { type RuleData } from "./EditRule";
import { steppedGradient } from "./ColorScaleRule";
import { getRuleDescription } from "./ruleDescription";
import "./conditional-formatting.css";

export interface Rule extends RuleData {
  id: string;
}

interface ConditionalFormattingProps {
  onClose: () => void;
  getSelectedArea: () => string;
  rules: Rule[];
  onRulesChange: (rules: Rule[]) => void;
  model: Model;
}

const ConditionalFormatting = ({
  onClose,
  getSelectedArea,
  rules,
  onRulesChange,
  model,
}: ConditionalFormattingProps) => {
  const [editingRule, setEditingRule] = useState<Rule | null>(null);
  const [isCreatingNew, setIsCreatingNew] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const { t } = useTranslation();

  const resolveRef = (val: string): string => {
    if (!val.trim()) return val;
    const parsed = parseRangeInSheet(model, val);
    if (parsed) {
      const [sheet, row, col] = parsed;
      const resolved = model.getFormattedCellValue(sheet, row, col);
      return resolved !== "" ? resolved : '""';
    }
    return val;
  };

  const isEditView = isCreatingNew || editingRule !== null;

  const handleCancel = () => {
    setIsCreatingNew(false);
    setEditingRule(null);
  };

  const handleSave = (data: RuleData) => {
    if (editingRule) {
      onRulesChange(
        rules.map((r) =>
          r.id === editingRule.id ? { ...data, id: editingRule.id } : r,
        ),
      );
      setEditingRule(null);
    } else {
      onRulesChange([...rules, { ...data, id: crypto.randomUUID() }]);
      setIsCreatingNew(false);
    }
  };

  const handleDelete = (id: string) => {
    onRulesChange(rules.filter((r) => r.id !== id));
  };

  if (isEditView) {
    const headerTitle = editingRule
      ? t("conditional_formatting.edit_rule")
      : t("conditional_formatting.add_new_rule");

    return (
      <div className="ic-cf-container">
        <div className="ic-cf-edit-header">
          <Tooltip title={t("conditional_formatting.back_to_list")}>
            <IconButton
              icon={<ArrowLeft />}
              onClick={handleCancel}
              aria-label={t("conditional_formatting.back_to_list")}
            />
          </Tooltip>
          <div className="ic-cf-edit-header-title">{headerTitle}</div>
          <Tooltip title={t("right_drawer.close")}>
            <IconButton
              icon={<X />}
              onClick={onClose}
              aria-label={t("right_drawer.close")}
            />
          </Tooltip>
        </div>
        <div className="ic-cf-content">
          <EditRule
            onSave={handleSave}
            onCancel={handleCancel}
            getSelectedArea={getSelectedArea}
            resolveValue={resolveRef}
            initialValues={
              editingRule ?? {
                applyTo: getSelectedArea(),
                ruleType: "cell_value",
                ruleOperator: "between",
                ruleValue: "",
                ruleValue2: "",
                formatStyle: {
                  bold: false,
                  italic: false,
                  underline: false,
                  strike: false,
                  fontColor: "#C0392B",
                  fillColor: "#FADBD8",
                },
              }
            }
          />
        </div>
      </div>
    );
  }

  const filteredRules = rules.filter((rule) => {
    if (!searchQuery.trim()) return true;
    const q = searchQuery.trim().toLowerCase();
    return (
      rule.applyTo.toLowerCase().includes(q) ||
      getRuleDescription({ ...rule, resolveValue: resolveRef })
        .toLowerCase()
        .includes(q)
    );
  });

  return (
    <div className="ic-cf-container">
      <div className="ic-cf-header">
        <div className="ic-cf-header-title">
          {t("conditional_formatting.title")}
        </div>
        <Tooltip title={t("right_drawer.close")}>
          <IconButton
            icon={<X />}
            onClick={onClose}
            aria-label={t("right_drawer.close")}
          />
        </Tooltip>
      </div>
      <div className="ic-cf-content">
        {rules.length === 0 ? (
          <div className="ic-cf-empty-state">
            <div className="ic-cf-icon-wrapper">
              <PackageOpen />
            </div>
            {t("conditional_formatting.empty_message1")}
            <br />
            {t("conditional_formatting.empty_message2")}
          </div>
        ) : (
          <div className="ic-cf-list-container">
            <div className="ic-cf-search-container">
              <Input
                type="text"
                size="sm"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder={t("conditional_formatting.search_placeholder")}
                startAdornment={<Search />}
              />
            </div>
            {/* biome-ignore lint/a11y/noStaticElementInteractions: prevents search input from losing focus on list click */}
            <div
              className="ic-cf-list-body"
              onMouseDown={(e) => e.preventDefault()}
            >
              {filteredRules.length === 0 ? (
                <div className="ic-cf-empty-state">
                  <div className="ic-cf-icon-wrapper">
                    <SearchX />
                  </div>
                  {t("conditional_formatting.no_search_results")}
                </div>
              ) : (
                filteredRules.map((rule) => {
                  const previewStyle: React.CSSProperties = {
                    color: rule.formatStyle.fontColor || "#000000",
                    backgroundColor:
                      rule.formatStyle.fillColor || "transparent",
                    fontWeight: rule.formatStyle.bold ? "bold" : "normal",
                    fontStyle: rule.formatStyle.italic ? "italic" : "normal",
                    textDecoration:
                      [
                        rule.formatStyle.underline ? "underline" : "",
                        rule.formatStyle.strike ? "line-through" : "",
                      ]
                        .filter(Boolean)
                        .join(" ") || "none",
                  };

                  const isColorScale = rule.ruleType === "color_scale";
                  const colorScaleGradient = isColorScale && rule.colorScale
                    ? steppedGradient([rule.colorScale.minimum.color, rule.colorScale.midpoint.color, rule.colorScale.maximum.color])
                    : undefined;

                  return (
                    // biome-ignore lint/a11y/noStaticElementInteractions: FIXME
                    <div
                      key={rule.id}
                      className="ic-cf-list-item"
                      // biome-ignore lint/a11y/noNoninteractiveTabindex: FIXME
                      tabIndex={0}
                      onClick={() => setEditingRule(rule)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter" || e.key === " ") {
                          e.preventDefault();
                          setEditingRule(rule);
                        }
                      }}
                    >
                      <div
                        className="ic-cf-list-item-preview"
                        style={isColorScale ? { background: colorScaleGradient } : previewStyle}
                      >
                        {!isColorScale && "Aa"}
                      </div>
                      <div className="ic-cf-list-item-text">
                        <div className="ic-cf-list-item-rule">
                          {getRuleDescription({
                            ...rule,
                            resolveValue: resolveRef,
                          })}
                        </div>
                        <div className="ic-cf-list-item-range">
                          {rule.applyTo || "—"}
                        </div>
                      </div>
                      <div className="ic-cf-list-item-icons">
                        <Tooltip title={t("conditional_formatting.edit_rule")}>
                          <IconButton
                            icon={<PencilLine />}
                            onClick={(e) => {
                              e.stopPropagation();
                              setEditingRule(rule);
                            }}
                            aria-label={t("conditional_formatting.edit_rule")}
                          />
                        </Tooltip>
                        <Tooltip
                          title={t("conditional_formatting.delete_rule")}
                        >
                          <IconButton
                            icon={<Trash2 />}
                            onClick={(e) => {
                              e.stopPropagation();
                              handleDelete(rule.id);
                            }}
                            aria-label={t("conditional_formatting.delete_rule")}
                          />
                        </Tooltip>
                      </div>
                    </div>
                  );
                })
              )}
            </div>
          </div>
        )}
      </div>
      <div className="ic-cf-footer">
        <Button startIcon={<Plus />} onClick={() => setIsCreatingNew(true)}>
          {t("conditional_formatting.new")}
        </Button>
      </div>
    </div>
  );
};

export default ConditionalFormatting;
