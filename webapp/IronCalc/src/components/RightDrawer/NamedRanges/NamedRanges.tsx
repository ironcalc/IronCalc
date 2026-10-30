import type {
  DefinedName,
  Model,
  NamedFunction,
  NamedFunctionInput,
} from "@ironcalc/wasm";
import {
  ArrowLeft,
  Copy,
  EyeOff,
  PackageOpen,
  PencilLine,
  Plus,
  Search,
  SearchX,
  SquareFunction,
  Tag,
  Trash2,
  X,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import { IconButton } from "../../Button/IconButton";
import { parseRangeInSheet } from "../../Editor/util";
import { Input } from "../../Input/Input";
import { Confirm } from "../../Modal";
import { Select } from "../../Select/Select";
import { ToggleButton } from "../../ToggleButton/ToggleButton";
import { Tooltip } from "../../Tooltip/Tooltip";
import EditNamedFunction from "../NamedFunctions/EditNamedFunction";
import { firstFreeName } from "../NamedFunctions/firstFreeName";
import EditNamedRange, {
  formatOnSaveError,
  type SaveError,
} from "./EditNamedRange";
import "./named-ranges.css";
import "../NamedFunctions/named-functions.css";

const normalizeRangeString = (range: string): string => {
  return range.trim().replace(/['"]/g, "");
};

/** The two kinds of names the drawer lists and edits */
export type NameKind = "range" | "function";

interface NamedRangesProps {
  onClose: () => void;
  model: Model;
  getSelectedArea: () => string;
  onUpdate: () => void;
  /** The kind of names listed when the drawer opens */
  initialKind: NameKind;
}

type View =
  | { kind: "list" }
  | { kind: "new"; type: NameKind }
  | { kind: "editRange"; editing: DefinedName }
  | { kind: "editFunction"; editing: NamedFunction };

const toFunctionInput = (f: NamedFunction): NamedFunctionInput => ({
  name: f.name,
  scope: f.scope ?? null,
  hidden: f.hidden,
  description: f.description,
  parameters: f.parameters.map((p) => ({
    name: p.name,
    description: p.description,
    optional: p.optional,
  })),
  formula: f.formula,
});

/** The named ranges and the named functions of the workbook */
const NamedRanges = ({
  onClose,
  getSelectedArea,
  model,
  onUpdate,
  initialKind,
}: NamedRangesProps) => {
  const [view, setView] = useState<View>({ kind: "list" });
  // Which kinds are listed. There is always at least one.
  const [showRanges, setShowRanges] = useState(initialKind === "range");
  const [showFunctions, setShowFunctions] = useState(
    initialKind === "function",
  );
  const [scopeFilter, setScopeFilter] = useState("[all]");
  // Hidden names are left out of the list (and the editor's "Advanced"
  // section) unless the user opts in
  const [includeHidden, setIncludeHidden] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [deletingFunction, setDeletingFunction] =
    useState<NamedFunction | null>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const { t } = useTranslation();

  const isListView = view.kind === "list";
  useEffect(() => {
    if (isListView) {
      searchRef.current?.focus();
    }
  }, [isListView]);

  const editingDefinedName = view.kind === "editRange" ? view.editing : null;
  const isCreatingNew = view.kind === "new";

  const handleCancel = () => setView({ kind: "list" });

  const handleSave = (
    name: string,
    scope: string,
    formula: string,
    hidden: boolean,
    comment: string,
  ): SaveError => {
    const worksheets = model.getWorksheetsProperties();
    if (isCreatingNew) {
      const scope_index = worksheets.findIndex((s) => s.name === scope);
      const newScope = scope_index >= 0 ? scope_index : null;
      try {
        model.newDefinedName({
          name,
          scope: newScope,
          formula,
          hidden,
          comment,
        });
        // Make sure the saved name is listed
        setShowRanges(true);
        setView({ kind: "list" });
        onUpdate();
        return {
          formulaError: "",
          nameError: "",
        };
      } catch (e) {
        if (e instanceof Error) {
          return formatOnSaveError(e.message);
        }
        return { formulaError: "", nameError: `${e}` };
      }
    } else {
      if (!editingDefinedName) {
        return {
          formulaError: "",
          nameError: "",
        };
      }

      const scope_index = worksheets.findIndex((s) => s.name === scope);
      const newScope = scope_index >= 0 ? scope_index : null;
      try {
        model.updateDefinedName(
          editingDefinedName.name,
          editingDefinedName.scope ?? null,
          { name, scope: newScope, formula, hidden, comment },
        );
        // Make sure the saved name is listed
        setShowRanges(true);
        setView({ kind: "list" });
        onUpdate();
        return { formulaError: "", nameError: "" };
      } catch (e) {
        if (e instanceof Error) {
          return formatOnSaveError(e.message);
        }
        return { formulaError: "", nameError: `${e}` };
      }
    }
  };

  const handleSaveFunction = (entry: NamedFunctionInput): string | null => {
    try {
      if (view.kind === "editFunction") {
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
    // Make sure the saved function is listed
    setShowFunctions(true);
    setView({ kind: "list" });
    onUpdate();
    return null;
  };

  const handleDuplicateFunction = (f: NamedFunction) => {
    const taken = model
      .getNamedFunctionList()
      .filter((g) => (g.scope ?? null) === (f.scope ?? null))
      .map((g) => g.name);
    const entry = { ...toFunctionInput(f), name: firstFreeName(f.name, taken) };
    try {
      model.newNamedFunction(entry);
      onUpdate();
    } catch (e) {
      console.error(e);
    }
  };

  const handleDeleteFunction = () => {
    if (!deletingFunction) {
      return;
    }
    model.deleteNamedFunction(
      deletingFunction.name,
      deletingFunction.scope ?? null,
    );
    setDeletingFunction(null);
    onUpdate();
  };

  // The editor of the name being edited or created
  if (view.kind !== "list") {
    const isFunction =
      view.kind === "editFunction" ||
      (view.kind === "new" && view.type === "function");

    let name = "";
    let scopeName = "[Global]";
    let formula = "";
    let comment = "";
    let hidden = false;

    if (editingDefinedName) {
      name = editingDefinedName.name;
      comment = editingDefinedName.comment;
      hidden = editingDefinedName.hidden;
      const worksheets = model.getWorksheetsProperties();
      scopeName =
        editingDefinedName.scope != null
          ? worksheets[editingDefinedName.scope]?.name || "[unknown]"
          : "[Global]";
      formula = editingDefinedName.formula;
    } else if (isCreatingNew) {
      formula = getSelectedArea();
    }

    const headerTitle = {
      new: isFunction
        ? t("named_functions.add_new_function")
        : t("name_manager_dialog.add_new_range"),
      editRange: t("name_manager_dialog.edit_range"),
      editFunction: t("named_functions.edit_function"),
    }[view.kind];

    return (
      <div className="ic-named-ranges-container">
        <div className="ic-named-ranges-edit-header">
          <Tooltip title={t("name_manager_dialog.back_to_list")}>
            <IconButton
              icon={<ArrowLeft />}
              onClick={handleCancel}
              aria-label={t("name_manager_dialog.back_to_list")}
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
          {/* A new name is a range or a function; an existing one can't change */}
          {view.kind === "new" ? (
            <div className="ic-named-ranges-kind-toggle">
              <ToggleButton
                fullWidth
                value={view.type}
                options={[
                  {
                    value: "range",
                    label: t("name_manager_dialog.kind_range"),
                  },
                  {
                    value: "function",
                    label: t("name_manager_dialog.kind_function"),
                  },
                ]}
                onChange={(type) => setView({ kind: "new", type })}
              />
            </div>
          ) : null}
          <div className="ic-named-ranges-editor">
            {isFunction ? (
              <EditNamedFunction
                model={model}
                editing={view.kind === "editFunction" ? view.editing : null}
                initial={
                  view.kind === "editFunction"
                    ? toFunctionInput(view.editing)
                    : null
                }
                showAdvanced={includeHidden}
                onSave={handleSaveFunction}
                onCancel={handleCancel}
              />
            ) : (
              <EditNamedRange
                name={name}
                scope={scopeName}
                formula={formula}
                comment={comment}
                hidden={hidden}
                showAdvanced={includeHidden}
                onSave={handleSave}
                onCancel={handleCancel}
                editingDefinedName={editingDefinedName}
                model={model}
              />
            )}
          </div>
        </div>
      </div>
    );
  }

  const currentSelectedArea = getSelectedArea();
  const definedNameList = model.getDefinedNameList();
  const functionList = model.getNamedFunctionList();
  const worksheets = model.getWorksheetsProperties();
  const currentSheetIndex = model.getSelectedView().sheet;
  const scopeOptions = [
    { value: "[all]", label: t("name_manager_dialog.scope_filter_all") },
    { value: "[global]", label: t("name_manager_dialog.scope_filter_global") },
    {
      value: "[current]",
      label: t("name_manager_dialog.scope_filter_current"),
    },
  ];
  const query = searchQuery.trim().toLowerCase();
  const scopeName = (scope: number | null | undefined) =>
    scope != null
      ? (worksheets[scope]?.name ?? "")
      : t("name_manager_dialog.scope_filter_global");
  const inScope = (scope: number | null | undefined) =>
    scopeFilter === "[all]" ||
    (scopeFilter === "[global]" && scope == null) ||
    (scopeFilter === "[current]" && scope === currentSheetIndex);
  const filteredDefinedNameList = showRanges
    ? definedNameList.filter(
        (definedName) =>
          (includeHidden || !definedName.hidden) &&
          inScope(definedName.scope) &&
          (!query ||
            definedName.name.toLowerCase().includes(query) ||
            scopeName(definedName.scope).toLowerCase().includes(query)),
      )
    : [];
  const filteredFunctionList = showFunctions
    ? functionList.filter(
        (f) =>
          (includeHidden || !f.hidden) &&
          inScope(f.scope) &&
          (!query ||
            f.name.toLowerCase().includes(query) ||
            f.description.toLowerCase().includes(query) ||
            f.parameters.some(
              (p) =>
                p.name.toLowerCase().includes(query) ||
                p.description.toLowerCase().includes(query),
            ) ||
            scopeName(f.scope).toLowerCase().includes(query)),
      )
    : [];
  const onlyFunctions = showFunctions && !showRanges;
  const onlyRanges = showRanges && !showFunctions;
  const isFiltered = scopeFilter !== "[all]" || query !== "";
  const listedCount =
    (showRanges ? definedNameList.length : 0) +
    (showFunctions ? functionList.length : 0);
  // Why the list is empty, if it is
  let emptyMessage: { icon: React.ReactNode; text: React.ReactNode } | null =
    null;
  if (filteredDefinedNameList.length + filteredFunctionList.length === 0) {
    if (isFiltered && listedCount > 0) {
      emptyMessage = {
        icon: <SearchX />,
        text: onlyFunctions
          ? t("named_functions.no_search_results", {
              query: searchQuery.trim(),
            })
          : onlyRanges
            ? t("name_manager_dialog.no_search_results")
            : t("name_manager_dialog.no_search_results_all"),
      };
    } else if (listedCount > 0) {
      // Every name is hidden and hidden names are excluded: tell the user
      // how to see them
      emptyMessage = {
        icon: <EyeOff />,
        text: t("name_manager_dialog.only_hidden_names"),
      };
    } else {
      emptyMessage = {
        icon: onlyFunctions ? <SquareFunction /> : <PackageOpen />,
        text: (
          <>
            {onlyFunctions
              ? t("named_functions.empty_message1")
              : t("name_manager_dialog.empty_message1")}
            <br />
            {t("name_manager_dialog.empty_message2")}
          </>
        ),
      };
    }
  }
  const title = onlyFunctions
    ? t("named_functions.title")
    : onlyRanges
      ? t("name_manager_dialog.title")
      : t("name_manager_dialog.title_all");
  const onNameSelected = (formula: string) => {
    const range = parseRangeInSheet(model, formula);
    if (range) {
      const [sheetIndex, rowStart, columnStart, rowEnd, columnEnd] = range;
      model.setSelectedSheet(sheetIndex);
      model.setSelectedCell(rowStart, columnStart);
      model.setSelectedRange(rowStart, columnStart, rowEnd, columnEnd);
    }
    onUpdate();
  };

  return (
    <div className="ic-named-ranges-container">
      <div className="ic-named-ranges-header">
        <div className="ic-named-ranges-header-title">{title}</div>
        <Tooltip title={t("right_drawer.close")}>
          <IconButton
            icon={<X />}
            onClick={onClose}
            aria-label={t("right_drawer.close")}
          />
        </Tooltip>
      </div>
      <div className="ic-named-ranges-content">
        {definedNameList.length + functionList.length === 0 ? (
          <div className="ic-named-ranges-empty-state-message">
            <div className="ic-named-ranges-icon-wrapper">
              {emptyMessage?.icon}
            </div>
            {emptyMessage?.text}
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
                placeholder={t("name_manager_dialog.search_placeholder")}
                startAdornment={<Search />}
              />
              <div className="ic-named-ranges-scope-filter">
                <span>{t("name_manager_dialog.scope_label")}</span>
                <Select
                  size="sm"
                  variant="ghost"
                  value={scopeFilter}
                  options={scopeOptions}
                  onChange={setScopeFilter}
                  toggles={[
                    // Unchecking the only kind listed lists the other one
                    {
                      value: "[ranges]",
                      label: t("name_manager_dialog.title"),
                      checked: showRanges,
                      onChange: (checked) => {
                        setShowRanges(checked);
                        setShowFunctions(showFunctions || !checked);
                      },
                    },
                    {
                      value: "[functions]",
                      label: t("named_functions.title"),
                      checked: showFunctions,
                      onChange: (checked) => {
                        setShowFunctions(checked);
                        setShowRanges(showRanges || !checked);
                      },
                    },
                    {
                      value: "[hidden]",
                      label: t("name_manager_dialog.include_hidden"),
                      checked: includeHidden,
                      onChange: setIncludeHidden,
                    },
                  ]}
                />
              </div>
            </div>
            {/* biome-ignore lint/a11y/noStaticElementInteractions: prevents search input from losing focus on list click */}
            <div
              className="ic-named-ranges-list-body"
              onMouseDown={(e) => e.preventDefault()}
            >
              {emptyMessage ? (
                <div className="ic-named-ranges-empty-state-message">
                  <div className="ic-named-ranges-icon-wrapper">
                    {emptyMessage.icon}
                  </div>
                  {emptyMessage.text}
                </div>
              ) : null}
              {filteredDefinedNameList.map((definedName) => {
                const isSelected =
                  currentSelectedArea !== null &&
                  normalizeRangeString(definedName.formula) ===
                    normalizeRangeString(currentSelectedArea);
                return (
                  // biome-ignore lint/a11y/noStaticElementInteractions: FIXME
                  <div
                    className={`ic-named-ranges-list-item ${isSelected ? "ic-named-ranges-list-item--selected" : ""} ${definedName.hidden ? "ic-named-ranges-list-item--hidden" : ""}`}
                    key={`${definedName.name}-${definedName.scope}`}
                    // biome-ignore lint/a11y/noNoninteractiveTabindex: FIXME
                    tabIndex={0}
                    onClick={() => {
                      // select the area corresponding to the defined name
                      const formula = definedName.formula;
                      const range = parseRangeInSheet(model, formula);
                      if (range) {
                        const [
                          sheetIndex,
                          rowStart,
                          columnStart,
                          rowEnd,
                          columnEnd,
                        ] = range;
                        model.setSelectedSheet(sheetIndex);
                        model.setSelectedCell(rowStart, columnStart);
                        model.setSelectedRange(
                          rowStart,
                          columnStart,
                          rowEnd,
                          columnEnd,
                        );
                      }
                      onUpdate();
                    }}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" || e.key === " ") {
                        e.preventDefault();
                        onNameSelected(definedName.formula);
                      }
                    }}
                  >
                    <div className="ic-named-ranges-list-item-text">
                      <div className="ic-named-ranges-name-text">
                        <span className="ic-named-ranges-kind-icon">
                          <Tag aria-label={t("name_manager_dialog.title")} />
                        </span>
                        {definedName.name}
                        {definedName.hidden ? (
                          <Tooltip title={t("name_manager_dialog.hidden")}>
                            <span className="ic-named-ranges-hidden-icon">
                              <EyeOff
                                aria-label={t("name_manager_dialog.hidden")}
                              />
                            </span>
                          </Tooltip>
                        ) : null}
                      </div>
                      <div className="ic-named-ranges-scope-text">
                        {definedName.scope != null
                          ? worksheets[definedName.scope]?.name || "[Unknown]"
                          : "[Global]"}
                      </div>
                      <div className="ic-named-ranges-formula-text">
                        {definedName.formula}
                      </div>
                      {definedName.comment ? (
                        <div className="ic-named-ranges-comment-text">
                          {definedName.comment}
                        </div>
                      ) : null}
                    </div>

                    <div className="ic-named-ranges-icons-wrapper">
                      <Tooltip title={t("name_manager_dialog.edit")}>
                        <IconButton
                          icon={<PencilLine />}
                          onClick={(e) => {
                            e.stopPropagation();
                            setView({
                              kind: "editRange",
                              editing: definedName,
                            });
                          }}
                          aria-label={t("name_manager_dialog.edit")}
                        />
                      </Tooltip>
                      <Tooltip title={t("name_manager_dialog.delete")}>
                        <IconButton
                          icon={<Trash2 />}
                          onClick={(e) => {
                            e.stopPropagation();
                            model.deleteDefinedName(
                              definedName.name,
                              definedName.scope ?? null,
                            );
                            onUpdate();
                          }}
                          aria-label={t("name_manager_dialog.delete")}
                        />
                      </Tooltip>
                    </div>
                  </div>
                );
              })}
              {filteredFunctionList.map((f) => (
                <div
                  className={`ic-named-ranges-list-item ic-named-functions-list-item ${f.hidden ? "ic-named-ranges-list-item--hidden" : ""}`}
                  key={`function-${f.name}-${f.scope}`}
                >
                  <div className="ic-named-ranges-list-item-text">
                    <div className="ic-named-ranges-name-text">
                      <span className="ic-named-ranges-kind-icon">
                        <SquareFunction
                          aria-label={t("named_functions.title")}
                        />
                      </span>
                      {f.name}({f.parameters.map((p) => p.name).join(", ")})
                      {f.hidden ? (
                        <Tooltip title={t("name_manager_dialog.hidden")}>
                          <span className="ic-named-ranges-hidden-icon">
                            <EyeOff
                              aria-label={t("name_manager_dialog.hidden")}
                            />
                          </span>
                        </Tooltip>
                      ) : null}
                    </div>
                    <div className="ic-named-ranges-scope-text">
                      {f.scope != null
                        ? worksheets[f.scope]?.name || "[Unknown]"
                        : "[Global]"}
                    </div>
                    <div className="ic-named-ranges-formula-text">
                      {f.formula}
                    </div>
                    {f.description ? (
                      <div className="ic-named-ranges-comment-text">
                        {f.description}
                      </div>
                    ) : null}
                  </div>
                  <div className="ic-named-ranges-icons-wrapper">
                    <Tooltip title={t("named_functions.edit")}>
                      <IconButton
                        icon={<PencilLine />}
                        onClick={() =>
                          setView({ kind: "editFunction", editing: f })
                        }
                        aria-label={t("named_functions.edit")}
                      />
                    </Tooltip>
                    <Tooltip title={t("named_functions.duplicate")}>
                      <IconButton
                        icon={<Copy />}
                        onClick={() => handleDuplicateFunction(f)}
                        aria-label={t("named_functions.duplicate")}
                      />
                    </Tooltip>
                    <Tooltip title={t("named_functions.delete")}>
                      <IconButton
                        icon={<Trash2 />}
                        onClick={() => setDeletingFunction(f)}
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
          onClick={() =>
            setView({ kind: "new", type: onlyFunctions ? "function" : "range" })
          }
        >
          {t("name_manager_dialog.new")}
        </Button>
      </div>
      <Confirm
        open={deletingFunction !== null}
        onClose={() => setDeletingFunction(null)}
        onConfirm={handleDeleteFunction}
        title={t("named_functions.delete_title", {
          name: deletingFunction?.name,
        })}
        message={
          <>
            {t("named_functions.delete_message", {
              name: deletingFunction?.name,
            })}
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

export default NamedRanges;
