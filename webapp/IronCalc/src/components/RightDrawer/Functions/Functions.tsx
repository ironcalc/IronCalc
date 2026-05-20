import type { Model } from "@ironcalc/wasm";
import { Search, SearchX, SquareArrowRightEnter, X } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { IconButton } from "../../Button/IconButton";
import { Input } from "../../Input/Input";
import { Select } from "../../Select/Select";
import { Tooltip } from "../../Tooltip/Tooltip";
import type { WorkbookState } from "../../workbookState";
import functionsDb from "./functions-db.json";
import "./functions.css";

type FunctionEntry = {
  c: string;
  i?: boolean;
  d: string;
  a: string[];
};

const DB = functionsDb as Record<string, FunctionEntry>;
const FUNCTIONS = Object.keys(DB).sort();
const COMMONLY_USED = ["SUM", "AVERAGE", "COUNT", "MAX", "MIN"];
const CATEGORIES = [
  "[all]",
  ...Array.from(new Set(FUNCTIONS.map((name) => DB[name].c))).sort(),
];

const buildSyntax = (name: string, args: string[]): string => {
  const formatted = args.map((a) =>
    a.endsWith("*") ? `[${a.slice(0, -1)}]` : a,
  );
  return `${name}(${formatted.join(", ")})`;
};

type FunctionsProps = {
  onClose: () => void;
  model: Model;
  workbookState: WorkbookState;
  onUpdate: () => void;
};

const Functions = ({
  onClose,
  model,
  workbookState,
  onUpdate,
}: FunctionsProps) => {
  const { t } = useTranslation();
  const [searchQuery, setSearchQuery] = useState("");
  const [categoryFilter, setCategoryFilter] = useState("[all]");
  const [expandedName, setExpandedName] = useState<string | null>(null);
  const isFiltering = searchQuery !== "" || categoryFilter !== "[all]";

  const handleInsert = (name: string) => {
    const { sheet, row, column } = model.getSelectedView();
    const text = `=${name}(`;
    workbookState.setEditingCell({
      sheet,
      row,
      column,
      text,
      cursorStart: text.length,
      cursorEnd: text.length,
      referencedRange: null,
      focus: "cell",
      activeRanges: [],
      mode: "accept",
      editorWidth: model.getColumnWidth(sheet, column),
      editorHeight: model.getRowHeight(sheet, row),
    });
    onUpdate();
  };

  const filteredFunctions = FUNCTIONS.filter((name) => {
    if (categoryFilter !== "[all]" && DB[name].c !== categoryFilter) {
      return false;
    }
    return name.toLowerCase().includes(searchQuery.toLowerCase());
  });

  const categoryOptions = CATEGORIES.map((c) => ({
    value: c,
    label: c === "[all]" ? t("functions.filter_all") : c,
  }));

  const renderItem = (name: string) => {
    const entry = DB[name];
    const isOpen = expandedName === name;
    return (
      // biome-ignore lint/a11y/noStaticElementInteractions: FIXME
      <div
        key={name}
        className={`ic-functions-list-item${isOpen ? " ic-functions-list-item--open" : ""}`}
        // biome-ignore lint/a11y/noNoninteractiveTabindex: FIXME
        tabIndex={0}
        onClick={() => setExpandedName(isOpen ? null : name)}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            setExpandedName(isOpen ? null : name);
          }
        }}
      >
        <div className="ic-functions-list-item-header">
          <span>{name}</span>
          {entry.i === false && (
            <span className="ic-functions-not-implemented">
              {t("functions.not_implemented")}
            </span>
          )}
          {entry.i !== false && (
            <div className="ic-functions-list-item-insert">
              <Tooltip title={t("functions.insert_function")}>
                <IconButton
                  icon={<SquareArrowRightEnter />}
                  aria-label={t("functions.insert_function")}
                  onClick={(e) => {
                    e.stopPropagation();
                    handleInsert(name);
                  }}
                />
              </Tooltip>
            </div>
          )}
        </div>
        {isOpen && (
          <div className="ic-functions-detail">
            <div className="ic-functions-detail-description">{entry.d}</div>
            <div className="ic-functions-detail-syntax">
              {buildSyntax(name, entry.a)}
            </div>
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="ic-functions-container">
      <div className="ic-functions-header">
        <div className="ic-functions-header-title">{t("functions.title")}</div>
        <Tooltip title={t("right_drawer.close")}>
          <IconButton
            icon={<X />}
            onClick={onClose}
            aria-label={t("right_drawer.close")}
          />
        </Tooltip>
      </div>
      <div className="ic-functions-search-container">
        <Input
          type="text"
          size="sm"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder={t("functions.search_placeholder")}
          startAdornment={<Search />}
        />
        <div className="ic-functions-category-filter">
          <Select
            size="sm"
            variant="ghost"
            value={categoryFilter}
            options={categoryOptions}
            onChange={setCategoryFilter}
          />
        </div>
      </div>
      <div className="ic-functions-list">
        {!isFiltering && (
          <>
            <div className="ic-functions-section-header">
              {t("functions.commonly_used")}
            </div>
            {COMMONLY_USED.map(renderItem)}
            <div className="ic-functions-divider" />
          </>
        )}
        {isFiltering && filteredFunctions.length === 0 ? (
          <div className="ic-functions-empty-state">
            <div className="ic-functions-icon-wrapper">
              <SearchX />
            </div>
            {t("functions.no_search_results")}
          </div>
        ) : (
          <>
            {!isFiltering && (
              <div className="ic-functions-section-header">
                {t("functions.all_functions")}
              </div>
            )}
            {filteredFunctions.map(renderItem)}
          </>
        )}
      </div>
    </div>
  );
};

export default Functions;
