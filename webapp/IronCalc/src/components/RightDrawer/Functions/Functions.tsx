import { Search, SearchX, X } from "lucide-react";
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { IconButton } from "../../Button/IconButton";
import { Input } from "../../Input/Input";
import { Tooltip } from "../../Tooltip/Tooltip";
import functionsDb from "./functions-db.json";
import "./functions.css";

type FunctionEntry = {
  i?: boolean;
  d: string;
  a: string[];
};

const DB = functionsDb as Record<string, FunctionEntry>;
const FUNCTIONS = Object.keys(DB);

const buildSyntax = (name: string, args: string[]): string => {
  const formatted = args.map((a) =>
    a.endsWith("*") ? `[${a.slice(0, -1)}]` : a,
  );
  return `${name}(${formatted.join(", ")})`;
};

type FunctionsProps = {
  onClose: () => void;
};

const Functions = ({ onClose }: FunctionsProps) => {
  const { t } = useTranslation();
  const [searchQuery, setSearchQuery] = useState("");
  const [expandedName, setExpandedName] = useState<string | null>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  const filteredFunctions = FUNCTIONS.filter((name) =>
    name.toLowerCase().includes(searchQuery.toLowerCase()),
  );

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
          ref={searchRef}
          type="text"
          size="sm"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder={t("functions.search_placeholder")}
          startAdornment={<Search />}
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
          filteredFunctions.map((name) => {
            const entry = DB[name];
            const isOpen = expandedName === name;
            return (
              // biome-ignore lint/a11y/noStaticElementInteractions: FIXME
              <div
                key={name}
                className={`ic-functions-list-item${isOpen ? " ic-functions-list-item--open" : ""}`}
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
                </div>
                {isOpen && (
                  <div className="ic-functions-detail">
                    <div className="ic-functions-detail-description">
                      {entry.d}
                    </div>
                    <div className="ic-functions-detail-syntax">
                      {buildSyntax(name, entry.a)}
                    </div>
                  </div>
                )}
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};

export default Functions;
