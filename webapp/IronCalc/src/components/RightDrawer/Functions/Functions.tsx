import { Search, SearchX, X } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { IconButton } from "../../Button/IconButton";
import { Input } from "../../Input/Input";
import { Select } from "../../Select/Select";
import { Tooltip } from "../../Tooltip/Tooltip";
import functionsDb from "./functions-db.json";
import "./functions.css";

type FunctionEntry = {
  c: string;
  d: string;
  a: string[];
};

const DB = functionsDb as Record<string, FunctionEntry>;
const FUNCTIONS = Object.keys(DB).sort();
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

const buildDocsUrl = (name: string): string => {
  const category = DB[name].c.toLowerCase().replace(/ /g, "_");
  return `https://docs.ironcalc.com/functions/${category}/${name.toLowerCase()}.html`;
};

type FunctionsProps = {
  onClose: () => void;
};

const Functions = ({ onClose }: FunctionsProps) => {
  const { t } = useTranslation();
  const [searchQuery, setSearchQuery] = useState("");
  const [categoryFilter, setCategoryFilter] = useState("[all]");
  const [expandedName, setExpandedName] = useState<string | null>(null);

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
      <div
        key={name}
        className={`ic-functions-list-item${isOpen ? " ic-functions-list-item--open" : ""}`}
      >
        <button
          type="button"
          className="ic-functions-list-item-header"
          onClick={() => setExpandedName(isOpen ? null : name)}
        >
          {name}
        </button>
        {isOpen && (
          <div className="ic-functions-detail">
            <div className="ic-functions-detail-description">{entry.d}</div>
            <div className="ic-functions-detail-syntax">
              {buildSyntax(name, entry.a)}
            </div>
            <a
              href={buildDocsUrl(name)}
              target="_blank"
              rel="noreferrer"
              className="ic-functions-read-more"
            >
              {t("functions.read_more")}
            </a>
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
        <Select
          className="ic-functions-category-filter"
          size="sm"
          variant="ghost"
          value={categoryFilter}
          options={categoryOptions}
          onChange={setCategoryFilter}
        />
      </div>
      <div className="ic-functions-list">
        {filteredFunctions.length === 0 ? (
          <div className="ic-functions-empty-state">
            <div className="ic-functions-icon-wrapper">
              <SearchX />
            </div>
            {t("functions.no_search_results")}
          </div>
        ) : (
          filteredFunctions.map(renderItem)
        )}
      </div>
    </div>
  );
};

export default Functions;
