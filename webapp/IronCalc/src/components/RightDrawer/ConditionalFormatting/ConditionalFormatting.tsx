import type { Dxf, IronCalcTheme, Model } from "@ironcalc/wasm";
import {
  ArrowLeft,
  ChevronDown,
  ChevronUp,
  Copy,
  PackageOpen,
  PencilLine,
  Plus,
  Search,
  SearchX,
  Trash2,
  X,
} from "lucide-react";
import { useRef, useState } from "react";
import { flushSync } from "react-dom";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import { IconButton } from "../../Button/IconButton";
import { resolveColorToHex } from "../../ColorPicker/util";
import {
  isRangeInRanges,
  isValidRanges,
  parseRangeInSheet,
  parseRect,
} from "../../Editor/util";
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

// Reorder animation, defined in conditional-formatting.css.
const MOVING_CLASS = "ic-cf-list-item--moving";
const MOVING_ANIMATION = "ic-cf-slide";
// Keep in sync with the animation duration in the CSS. Only used as a safety
// net in case the animation events never arrive.
const MOVING_DURATION_MS = 200;

interface ConditionalFormattingProps {
  onClose: () => void;
  getSelectedArea: () => string;
  sheet: number;
  onUpdate: () => void;
  model: Model;
  currentTheme: IronCalcTheme;
}

const ConditionalFormatting = ({
  onClose,
  getSelectedArea,
  sheet,
  onUpdate,
  model,
  currentTheme,
}: ConditionalFormattingProps) => {
  const [editingRule, setEditingRule] = useState<Rule | null>(null);
  const [isCreatingNew, setIsCreatingNew] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [filterOption, setFilterOption] = useState("sheet");
  const listBodyRef = useRef<HTMLDivElement>(null);
  // True while a reorder animation is running; further clicks are ignored
  // until the pending change has been committed.
  const isReordering = useRef(false);
  const { t } = useTranslation();

  function getSanitizedSelectedArea(): string {
    const selectedArea = getSelectedArea();
    // convert Sheet1!$A$1:$B$2 (or 'My Sheet'!$A$1:$B$2) to A1:B2
    const withoutSheet = selectedArea.split("!").pop() ?? selectedArea;
    return withoutSheet.replace(/\$/g, "");
  }

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
    const list = model.getConditionalFormattingList(sheet);
    return list.flatMap((cf) => {
      // List is priority-sorted; cf.index is the stable storage index.
      const modelIndex = cf.index;
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
          id: `${modelIndex}`,
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

  const selectRuleRange = (rule: Rule): void => {
    const firstRange = rule.applyTo.split(/\s+/)[0];
    const range = parseRect(firstRange);
    if (range) {
      const {
        minCol: columnStart,
        maxCol: columnEnd,
        minRow: rowStart,
        maxRow: rowEnd,
      } = range;
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
    const applyTo = data.applyTo;
    if (!isValidRanges(applyTo)) {
      return;
    }
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

  const handleDuplicate = (rule: Rule) => {
    const cfRule = ruleDataToCfRule(rule);
    if (!cfRule) {
      return;
    }
    model.addConditionalFormatting(sheet, rule.applyTo, cfRule);
    onUpdate();
  };

  const findRowElement = (id: string): HTMLElement | null =>
    listBodyRef.current?.querySelector<HTMLElement>(`[data-rule-id="${id}"]`) ??
    null;

  // Swapping two rules visually slides the two rows past each other with a
  // CSS animation. The model is only changed once the animation has ended,
  // so the redraw that follows puts the rows where they already appear to
  // be. The animation is a transform inside the scrollable list, so it is
  // clipped like any other content and touches nothing outside the widget.
  const handleReorder = (rule: Rule, lower: boolean) => {
    if (isReordering.current) {
      return;
    }
    const index = parseInt(rule.id, 10);
    const commit = () => {
      if (lower) {
        model.lowerConditionalFormattingPriority(sheet, index);
      } else {
        model.raiseConditionalFormattingPriority(sheet, index);
      }
      onUpdate();
    };

    const pos = rules.findIndex((r) => r.id === rule.id);
    const neighbor = rules[lower ? pos + 1 : pos - 1];
    const rowEl = findRowElement(rule.id);
    const neighborEl = neighbor ? findRowElement(neighbor.id) : null;
    const reducedMotion = window.matchMedia?.(
      "(prefers-reduced-motion: reduce)",
    ).matches;
    // No animation when the neighbor is filtered out of the list (nothing
    // moves on screen) or when the user prefers reduced motion.
    if (!rowEl || !neighborEl || reducedMotion) {
      commit();
      return;
    }

    const rowHeight = rowEl.getBoundingClientRect().height;
    const neighborHeight = neighborEl.getBoundingClientRect().height;
    const rowShift = lower ? neighborHeight : -neighborHeight;
    const neighborShift = lower ? -rowHeight : rowHeight;

    let finished = false;
    const finish = () => {
      if (finished) {
        return;
      }
      finished = true;
      clearTimeout(timer);
      rowEl.removeEventListener("animationend", onAnimationEvent);
      rowEl.removeEventListener("animationcancel", onAnimationEvent);
      isReordering.current = false;
      // Commit synchronously and drop the transforms in the same task, so the
      // DOM reorder and the transform removal are painted together.
      flushSync(commit);
      for (const el of [rowEl, neighborEl]) {
        el.classList.remove(MOVING_CLASS);
        el.style.removeProperty("--ic-cf-shift");
      }
    };
    const onAnimationEvent = (e: AnimationEvent) => {
      if (e.target === rowEl && e.animationName === MOVING_ANIMATION) {
        finish();
      }
    };
    rowEl.addEventListener("animationend", onAnimationEvent);
    rowEl.addEventListener("animationcancel", onAnimationEvent);
    const timer = setTimeout(finish, MOVING_DURATION_MS + 100);

    isReordering.current = true;
    rowEl.style.setProperty("--ic-cf-shift", `${rowShift}px`);
    neighborEl.style.setProperty("--ic-cf-shift", `${neighborShift}px`);
    rowEl.classList.add(MOVING_CLASS);
    neighborEl.classList.add(MOVING_CLASS);
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
            getSelectedArea={getSanitizedSelectedArea}
            resolveValue={resolveRef}
            currentTheme={currentTheme}
            initialValues={
              editingRule ?? {
                applyTo: getSanitizedSelectedArea(),
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
      if (!isRangeInRanges(getSanitizedSelectedArea(), rule.applyTo)) {
        return false;
      }
    }
    if (!searchQuery.trim()) {
      return true;
    }
    const q = searchQuery.trim().toLowerCase();
    return (
      rule.applyTo.toLowerCase().includes(q) ||
      getRuleDescription({ ...rule, resolveValue: resolveRef, t })
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
              ref={listBodyRef}
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
                  const isActive = isRangeInRanges(
                    getSanitizedSelectedArea(),
                    rule.applyTo,
                  );

                  const previewStyle: React.CSSProperties = {
                    color:
                      resolveColorToHex(
                        rule.formatStyle.fontColor,
                        currentTheme,
                      ) || "var(--palette-sheet-default-text-color)",
                    backgroundColor:
                      resolveColorToHex(
                        rule.formatStyle.fillColor,
                        currentTheme,
                      ) || "transparent",
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
                                resolveColorToHex(
                                  rule.colorScale.minimum.color,
                                  currentTheme,
                                ),
                                resolveColorToHex(
                                  rule.colorScale.maximum.color,
                                  currentTheme,
                                ),
                              ]
                            : [
                                resolveColorToHex(
                                  rule.colorScale.minimum.color,
                                  currentTheme,
                                ),
                                resolveColorToHex(
                                  rule.colorScale.midpoint.color,
                                  currentTheme,
                                ),
                                resolveColorToHex(
                                  rule.colorScale.maximum.color,
                                  currentTheme,
                                ),
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
                        color: resolveColorToHex(
                          rule.iconSets.rating.color,
                          currentTheme,
                        ),
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
                      data-rule-id={rule.id}
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
                      <div className="ic-cf-list-item-order-stack">
                        <IconButton
                          className="ic-cf-list-item-order-chevron"
                          icon={<ChevronUp />}
                          disabled={rules[0]?.id === rule.id}
                          onClick={(e) => {
                            e.stopPropagation();
                            handleReorder(rule, false);
                          }}
                          aria-label={t("conditional_formatting.move_up")}
                        />
                        <div className="ic-cf-list-item-order">
                          {rules.findIndex((r) => r.id === rule.id) + 1}
                        </div>
                        <IconButton
                          className="ic-cf-list-item-order-chevron"
                          icon={<ChevronDown />}
                          disabled={rules[rules.length - 1]?.id === rule.id}
                          onClick={(e) => {
                            e.stopPropagation();
                            handleReorder(rule, true);
                          }}
                          aria-label={t("conditional_formatting.move_down")}
                        />
                      </div>
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
                            color={
                              resolveColorToHex(
                                rule.dataBars.positiveColor,
                                currentTheme,
                              ) || rule.dataBars.color
                            }
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
                                t,
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
                          title={t("conditional_formatting.duplicate_rule")}
                        >
                          <IconButton
                            icon={<Copy />}
                            onClick={(e) => {
                              e.stopPropagation();
                              handleDuplicate(rule);
                            }}
                            aria-label={t(
                              "conditional_formatting.duplicate_rule",
                            )}
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
