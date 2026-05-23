import type {
  ConditionalFormatting as CfEntry,
  Dxf,
  Model,
} from "@ironcalc/wasm";
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
import { iconSpecFor } from "../../IconPicker/IconPicker";
import { Input } from "../../Input/Input";
import { Select } from "../../Select/Select";
import { Tooltip } from "../../Tooltip/Tooltip";
import { steppedGradient } from "./ColorScaleRule";
import {
  cfRuleToRuleData,
  dxfToFormatStyle,
  ruleDataToCfRule,
  ruleTypeUsesDxf,
} from "./cfRuleMapping";
import { DataBarMiniChart } from "./DataBarsRule";
import EditRule, { type RuleData } from "./EditRule";
import { ALL_PRESETS } from "./IconSetsRule";
import { getRuleDescription } from "./ruleDescription";
import "./conditional-formatting.css";

export interface Rule extends RuleData {
  id: string;
}

// HACK: We should just have the range without the sheet name in the model
function getRange(applyTo: string): string {
  const parts = applyTo.split("!");
  return parts.length === 2 ? parts[1] : applyTo;
}

function rangesIntersect(
  a: [number, number, number, number, number],
  b: [number, number, number, number, number],
): boolean {
  const [aSheet, aR1, aC1, aR2, aC2] = a;
  const [bSheet, bR1, bC1, bR2, bC2] = b;
  if (aSheet !== bSheet) {
    return false;
  }
  const aMinR = Math.min(aR1, aR2);
  const aMaxR = Math.max(aR1, aR2);
  const aMinC = Math.min(aC1, aC2);
  const aMaxC = Math.max(aC1, aC2);
  return aMinR <= bR2 && bR1 <= aMaxR && aMinC <= bC2 && bC1 <= aMaxC;
}

interface ConditionalFormattingProps {
  onClose: () => void;
  getSelectedArea: () => string;
  sheet: number;
  onUpdate: () => void;
  model: Model;
}

const ConditionalFormatting = ({
  onClose,
  getSelectedArea,
  sheet,
  onUpdate,
  model,
}: ConditionalFormattingProps) => {
  const [editingRule, setEditingRule] = useState<Rule | null>(null);
  const [isCreatingNew, setIsCreatingNew] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [filterOption, setFilterOption] = useState("sheet");
  const { t } = useTranslation();

  const filterOptions = [
    { value: "sheet", label: t("conditional_formatting.filter_this_sheet") },
    { value: "selection", label: t("conditional_formatting.filter_selection") },
  ];

  const resolveRef = (val: string): string => {
    if (!val.trim()) {
      return val;
    }
    const parsed = parseRangeInSheet(model, val);
    if (parsed) {
      const [sheetIdx, row, col] = parsed;
      const resolved = model.getFormattedCellValue(sheetIdx, row, col);
      return resolved !== "" ? resolved : '""';
    }
    return val;
  };

  const loadRules = (): Rule[] => {
    const list = model.getConditionalFormattingList(sheet) as CfEntry[];
    return list.flatMap((cf, modelIndex) => {
      const partial = cfRuleToRuleData(cf);
      if (!partial) {
        return [];
      }
      const ruleType = partial.ruleType ?? "cell_value";
      const dxf = ruleTypeUsesDxf(ruleType)
        ? (model.getDxfForConditionalFormatting(
            sheet,
            modelIndex,
          ) as Dxf | null)
        : null;
      const formatStyle = dxfToFormatStyle(dxf);
      return [
        {
          ...partial,
          id: String(modelIndex),
          applyTo: cf.range,
          ruleType,
          ruleOperator: partial.ruleOperator ?? "",
          ruleValue: partial.ruleValue ?? "",
          ruleValue2: partial.ruleValue2 ?? "",
          formatStyle,
        },
      ];
    });
  };

  const rules = loadRules();

  const sheetName = model.getWorksheetsProperties()[sheet]?.name ?? "";

  const getRuleRange = (rule: Rule): string =>
    rule.applyTo.includes("!") ? rule.applyTo : `${sheetName}!${rule.applyTo}`;

  const selectRuleRange = (rule: Rule): void => {
    const range = parseRangeInSheet(model, getRuleRange(rule));
    if (range) {
      const [sheetIndex, rowStart, columnStart, rowEnd, columnEnd] = range;
      model.setSelectedSheet(sheetIndex);
      model.setSelectedCell(rowStart, columnStart);
      model.setSelectedRange(rowStart, columnStart, rowEnd, columnEnd);
    }
    onUpdate();
  };

  const isEditView = isCreatingNew || editingRule !== null;

  const handleCancel = () => {
    setIsCreatingNew(false);
    setEditingRule(null);
  };

  const handleSave = (data: RuleData) => {
    const cfRule = ruleDataToCfRule(data);
    if (!cfRule) {
      return;
    }
    const applyTo = getRange(data.applyTo);
    if (editingRule) {
      model.updateConditionalFormatting(
        sheet,
        parseInt(editingRule.id, 10),
        applyTo,
        cfRule,
      );
      setEditingRule(null);
    } else {
      model.addConditionalFormatting(sheet, applyTo, cfRule);
      setIsCreatingNew(false);
    }
    onUpdate();
  };

  const handleDelete = (id: string) => {
    model.deleteConditionalFormatting(sheet, parseInt(id, 10));
    onUpdate();
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
                  fontColor: "#D21D21",
                  fillColor: "#FBE7E8",
                },
              }
            }
          />
        </div>
      </div>
    );
  }

  const filteredRules = rules.filter((rule) => {
    if (filterOption === "selection") {
      const selectedParsed = parseRangeInSheet(model, getSelectedArea());
      if (!selectedParsed) {
        return false;
      }
      const ruleParsed = parseRangeInSheet(model, getRuleRange(rule));
      if (!ruleParsed) {
        return false;
      }
      if (!rangesIntersect(selectedParsed, ruleParsed)) {
        return false;
      }
    }
    if (!searchQuery.trim()) {
      return true;
    }
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
              <div className="ic-cf-filter">
                <span>{t("conditional_formatting.filter_label")}</span>
                <Select
                  size="sm"
                  variant="ghost"
                  value={filterOption}
                  options={filterOptions}
                  onChange={setFilterOption}
                />
              </div>
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
                  const selectedParsed = parseRangeInSheet(
                    model,
                    getSelectedArea(),
                  );
                  const ruleParsed = parseRangeInSheet(
                    model,
                    getRuleRange(rule),
                  );
                  const isActive =
                    selectedParsed !== null &&
                    ruleParsed !== null &&
                    rangesIntersect(selectedParsed, ruleParsed);

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
                  const colorScaleGradient =
                    isColorScale && rule.colorScale
                      ? steppedGradient(
                          rule.colorScale.midpoint.type === "none"
                            ? [
                                rule.colorScale.minimum.color,
                                rule.colorScale.maximum.color,
                              ]
                            : [
                                rule.colorScale.minimum.color,
                                rule.colorScale.midpoint.color,
                                rule.colorScale.maximum.color,
                              ],
                        )
                      : undefined;
                  const isDataBars = rule.ruleType === "data_bars";
                  const isIconSets = rule.ruleType === "icon_sets";
                  const isRating = isIconSets && !!rule.iconSets?.rating;
                  const iconSetsFirstIcon = (() => {
                    if (!isIconSets || !rule.iconSets) {
                      return undefined;
                    }
                    if (rule.iconSets.rating) {
                      const spec = iconSpecFor(rule.iconSets.rating.icon);
                      return {
                        Icon: spec.Icon,
                        color: rule.iconSets.rating.color,
                        filled: spec.filled,
                        backendName: rule.iconSets.rating.icon,
                      };
                    }
                    return ALL_PRESETS.find(
                      (p) => p.id === rule.iconSets?.presetId,
                    )?.icons[0];
                  })();

                  return (
                    // biome-ignore lint/a11y/noStaticElementInteractions: FIXME
                    <div
                      key={rule.id}
                      className={`ic-cf-list-item${isActive ? " ic-cf-list-item--selected" : ""}`}
                      // biome-ignore lint/a11y/noNoninteractiveTabindex: FIXME
                      tabIndex={0}
                      onClick={() => selectRuleRange(rule)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter" || e.key === " ") {
                          e.preventDefault();
                          selectRuleRange(rule);
                        }
                      }}
                    >
                      <div
                        className="ic-cf-list-item-preview"
                        style={
                          isColorScale
                            ? { background: colorScaleGradient }
                            : isDataBars
                              ? { position: "relative", overflow: "hidden" }
                              : isIconSets
                                ? { background: "var(--palette-common-white)" }
                                : previewStyle
                        }
                      >
                        {isDataBars && rule.dataBars ? (
                          <DataBarMiniChart
                            color={rule.dataBars.color}
                            gradient={rule.dataBars.gradient}
                          />
                        ) : isIconSets && iconSetsFirstIcon ? (
                          <iconSetsFirstIcon.Icon
                            size={16}
                            color={iconSetsFirstIcon.color}
                            fill={
                              iconSetsFirstIcon.filled
                                ? iconSetsFirstIcon.color
                                : "none"
                            }
                          />
                        ) : (
                          !isColorScale && "Aa"
                        )}
                      </div>
                      <div className="ic-cf-list-item-text">
                        <div className="ic-cf-list-item-rule">
                          {isRating
                            ? t("conditional_formatting.icon_sets_ratings")
                            : getRuleDescription({
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
