import type { Model, NamedFunction, NamedFunctionInput } from "@ironcalc/wasm";
import {
  ArrowLeft,
  Copy,
  PencilLine,
  Plus,
  Search,
  SearchX,
  SquareFunction,
  Trash2,
  X,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import { IconButton } from "../../Button/IconButton";
import { Input } from "../../Input/Input";
import { Confirm } from "../../Modal";
import { Tooltip } from "../../Tooltip/Tooltip";
import EditNamedFunction from "./EditNamedFunction";
import { FormulaChips } from "./FormulaChips";
import { firstFreeName } from "./formulaSegments";
import "../NamedRanges/named-ranges.css";
import "../NamedRanges/edit-name-range.css";
import "./named-functions.css";

interface NamedFunctionsProps {
  onClose: () => void;
  model: Model;
  onUpdate: () => void;
}

type View =
  | { kind: "list" }
  | { kind: "new"; initial: NamedFunctionInput | null }
  | { kind: "edit"; editing: NamedFunction };

const toInput = (f: NamedFunction): NamedFunctionInput => ({
  name: f.name,
  scope: f.scope ?? null,
  description: f.description,
  parameters: f.parameters.map((p) => ({
    name: p.name,
    description: p.description,
    optional: p.optional,
  })),
  formula: f.formula,
});

const NamedFunctions = ({ onClose, model, onUpdate }: NamedFunctionsProps) => {
  const [view, setView] = useState<View>({ kind: "list" });
  const [searchQuery, setSearchQuery] = useState("");
  const [deleting, setDeleting] = useState<NamedFunction | null>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const { t } = useTranslation();

  const isListView = view.kind === "list";
  useEffect(() => {
    if (isListView) {
      searchRef.current?.focus();
    }
  }, [isListView]);

  const worksheets = model.getWorksheetsProperties();
  const scopeName = (f: NamedFunction) =>
    f.scope != null
      ? worksheets[f.scope]?.name || "[Unknown]"
      : t("name_manager_dialog.scope_filter_global");

  const handleSave = (entry: NamedFunctionInput): string | null => {
    try {
      if (view.kind === "edit") {
        model.updateNamedFunction(
          view.editing.name,
          view.editing.scope ?? null,
          entry,
        );
      } else {
        model.newNamedFunction(entry);
      }
    } catch (e) {
      return e instanceof Error ? e.message : `${e}`;
    }
    setView({ kind: "list" });
    onUpdate();
    return null;
  };

  const handleDuplicate = (f: NamedFunction) => {
    const taken = model
      .getNamedFunctionList()
      .filter((g) => (g.scope ?? null) === (f.scope ?? null))
      .map((g) => g.name);
    const entry = { ...toInput(f), name: firstFreeName(f.name, taken) };
    try {
      model.newNamedFunction(entry);
      onUpdate();
    } catch (e) {
      console.error(e);
    }
  };

  const handleDelete = () => {
    if (!deleting) {
      return;
    }
    model.deleteNamedFunction(deleting.name, deleting.scope ?? null);
    setDeleting(null);
    onUpdate();
  };

  if (view.kind !== "list") {
    const headerTitle =
      view.kind === "new"
        ? t("named_functions.add_new_function")
        : t("named_functions.edit_function");
    return (
      <div className="ic-named-ranges-container">
        <div className="ic-named-ranges-edit-header">
          <Tooltip title={t("named_functions.back_to_list")}>
            <IconButton
              icon={<ArrowLeft />}
              onClick={() => setView({ kind: "list" })}
              aria-label={t("named_functions.back_to_list")}
            />
          </Tooltip>
          <div className="ic-named-ranges-edit-header-title">{headerTitle}</div>
          <Tooltip title={t("right_drawer.close")}>
            <IconButton
              icon={<X />}
              onClick={onClose}
              aria-label={t("right_drawer.close")}
            />
          </Tooltip>
        </div>
        <div className="ic-named-ranges-content">
          <EditNamedFunction
            model={model}
            editing={view.kind === "edit" ? view.editing : null}
            initial={
              view.kind === "edit" ? toInput(view.editing) : view.initial
            }
            onSave={handleSave}
            onCancel={() => setView({ kind: "list" })}
          />
        </div>
      </div>
    );
  }

  const functionList = model.getNamedFunctionList();
  const query = searchQuery.trim().toLowerCase();
  const filteredList = query
    ? functionList.filter(
        (f) =>
          f.name.toLowerCase().includes(query) ||
          f.description.toLowerCase().includes(query) ||
          f.parameters.some(
            (p) =>
              p.name.toLowerCase().includes(query) ||
              p.description.toLowerCase().includes(query),
          ) ||
          scopeName(f).toLowerCase().includes(query),
      )
    : functionList;

  return (
    <div className="ic-named-ranges-container">
      <div className="ic-named-ranges-header">
        <div className="ic-named-ranges-header-title">
          {t("named_functions.title")}
        </div>
        <Tooltip title={t("right_drawer.close")}>
          <IconButton
            icon={<X />}
            onClick={onClose}
            aria-label={t("right_drawer.close")}
          />
        </Tooltip>
      </div>
      <div className="ic-named-ranges-content">
        {functionList.length === 0 ? (
          <div className="ic-named-ranges-empty-state-message">
            <div className="ic-named-ranges-icon-wrapper">
              <SquareFunction />
            </div>
            {t("named_functions.empty_message1")}
            <br />
            {t("named_functions.empty_message2")}
          </div>
        ) : (
          <div className="ic-named-ranges-list-container">
            <div className="ic-named-ranges-search-container">
              <Input
                ref={searchRef}
                type="text"
                size="sm"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder={t("named_functions.search_placeholder")}
                startAdornment={<Search />}
              />
            </div>
            <div className="ic-named-ranges-list-body">
              {filteredList.length === 0 ? (
                <div className="ic-named-ranges-empty-state-message">
                  <div className="ic-named-ranges-icon-wrapper">
                    <SearchX />
                  </div>
                  {t("named_functions.no_search_results", {
                    query: searchQuery.trim(),
                  })}
                </div>
              ) : null}
              {filteredList.map((f) => (
                <div
                  className="ic-named-ranges-list-item ic-named-functions-list-item"
                  key={`${f.name}-${f.scope}`}
                >
                  <div className="ic-named-ranges-list-item-text">
                    <div className="ic-named-ranges-name-text">
                      {f.name}({f.parameters.map((p) => p.name).join(", ")})
                    </div>
                    {f.scope != null ? (
                      <div className="ic-named-ranges-scope-text">
                        {scopeName(f)}
                      </div>
                    ) : null}
                    <div className="ic-named-ranges-formula-text">
                      <FormulaChips
                        formula={f.formula}
                        parameters={f.parameters.map((p) => p.name)}
                      />
                    </div>
                    {f.description ? (
                      <div className="ic-named-ranges-comment-text">
                        {f.description}
                      </div>
                    ) : null}
                  </div>
                  <div className="ic-named-ranges-icons-wrapper ic-named-functions-actions">
                    <Tooltip title={t("named_functions.edit")}>
                      <IconButton
                        icon={<PencilLine />}
                        onClick={() => setView({ kind: "edit", editing: f })}
                        aria-label={t("named_functions.edit")}
                      />
                    </Tooltip>
                    <Tooltip title={t("named_functions.duplicate")}>
                      <IconButton
                        icon={<Copy />}
                        onClick={() => handleDuplicate(f)}
                        aria-label={t("named_functions.duplicate")}
                      />
                    </Tooltip>
                    <Tooltip title={t("named_functions.delete")}>
                      <IconButton
                        icon={<Trash2 />}
                        onClick={() => setDeleting(f)}
                        aria-label={t("named_functions.delete")}
                      />
                    </Tooltip>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
      <div className="ic-named-ranges-footer">
        <Button
          startIcon={<Plus />}
          onClick={() => setView({ kind: "new", initial: null })}
        >
          {t("named_functions.new")}
        </Button>
      </div>
      <Confirm
        open={deleting !== null}
        onClose={() => setDeleting(null)}
        onConfirm={handleDelete}
        title={t("named_functions.delete_title", { name: deleting?.name })}
        message={
          <>
            {t("named_functions.delete_message", { name: deleting?.name })}
            <br />
            <strong>{t("named_functions.delete_consequence")}</strong>
          </>
        }
        confirmLabel={t("named_functions.delete_confirm")}
        cancelLabel={t("named_functions.cancel")}
        variant="destructive"
      />
    </div>
  );
};

export default NamedFunctions;
