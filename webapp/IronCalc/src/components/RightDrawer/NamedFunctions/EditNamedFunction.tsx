import type {
  Model,
  NamedFunction,
  NamedFunctionCheck,
  NamedFunctionInput,
  NamedFunctionParameterInput,
} from "@ironcalc/wasm";
import { Check, EyeOff, Info, Minus, Plus, SquareFunction } from "lucide-react";
import { useEffect, useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import { IconButton } from "../../Button/IconButton";
import { Checkbox } from "../../Checkbox/Checkbox";
import { Input } from "../../Input/Input";
import { Select } from "../../Select/Select";
import { Textarea } from "../../Textarea/Textarea";
import { Tooltip } from "../../Tooltip/Tooltip";
import { firstFreeName } from "./firstFreeName";

// Excel allows at most 255 characters in a defined name comment; the engine
// applies the same limit to descriptions
export const MAX_DESCRIPTION_LENGTH = 255;

const DEFAULT_NAME = "NEW_FUNCTION";

interface ParameterRow {
  key: number;
  name: string;
  description: string;
  /** Whether the user marked this argument as optional */
  optional: boolean;
}

interface EditNamedFunctionProps {
  model: Model;
  /** The function being edited, null when creating a new one */
  editing: NamedFunction | null;
  /** The initial values, the function being edited or the one to duplicate */
  initial: NamedFunctionInput | null;
  /** Whether the "Advanced" section (hide this function) is shown */
  showAdvanced: boolean;
  onSave: (entry: NamedFunctionInput) => string | null;
  onCancel: () => void;
}

const emptyCheck = (parameterCount: number): NamedFunctionCheck => ({
  name: null,
  parameters: new Array(parameterCount).fill(null),
  formula: [],
  warnings: [],
});

const EditNamedFunction = ({
  model,
  editing,
  initial,
  showAdvanced,
  onSave,
  onCancel,
}: EditNamedFunctionProps) => {
  const { t } = useTranslation();
  const worksheets = model.getWorksheetsProperties();

  const [name, setName] = useState(() => {
    if (initial) {
      return initial.name;
    }
    const taken = model.getNamedFunctionList().map((f) => f.name);
    return firstFreeName(DEFAULT_NAME, taken);
  });
  // The scope is tracked by sheet id so it survives sheet renames
  const [scopeSheetId, setScopeSheetId] = useState<number | undefined>(() =>
    initial?.scope != null ? worksheets[initial.scope]?.sheet_id : undefined,
  );
  const scopeIndex = worksheets.findIndex((s) => s.sheet_id === scopeSheetId);
  const scope = scopeIndex >= 0 ? scopeIndex : null;
  const scopeName = scopeIndex >= 0 ? worksheets[scopeIndex].name : "[Global]";
  const [hidden, setHidden] = useState(initial?.hidden ?? false);
  const [description, setDescription] = useState(initial?.description ?? "");
  const [parameters, setParameters] = useState<ParameterRow[]>(() =>
    (initial?.parameters ?? []).map((p, key) => ({
      key,
      name: p.name,
      description: p.description ?? "",
      optional: p.optional ?? false,
    })),
  );
  const [nextKey, setNextKey] = useState(parameters.length);
  const [formula, setFormula] = useState(initial?.formula ?? "");
  const [check, setCheck] = useState<NamedFunctionCheck>(() =>
    emptyCheck(parameters.length),
  );
  const [saveError, setSaveError] = useState("");

  const nameId = useId();
  const formulaId = useId();

  // Arguments are matched by position, so the optional ones go last: every
  // argument after an optional one is optional too
  const firstOptional = parameters.findIndex((p) => p.optional);
  const isForcedOptional = (index: number) =>
    firstOptional !== -1 && index > firstOptional;

  const toInput = (): NamedFunctionInput => ({
    name: name.trim(),
    scope,
    hidden,
    description,
    parameters: parameters.map(
      (p, index): NamedFunctionParameterInput => ({
        name: p.name.trim(),
        description: p.description,
        optional: p.optional || isForcedOptional(index),
      }),
    ),
    formula,
  });

  // The engine checks the function, field by field, as the user types
  // biome-ignore lint/correctness/useExhaustiveDependencies: `toInput` is derived from the listed state
  useEffect(() => {
    const timer = setTimeout(() => {
      const result = model.checkNamedFunction(
        toInput(),
        editing?.name ?? null,
        editing?.scope ?? null,
      );
      setCheck(result);
    }, 300);
    return () => clearTimeout(timer);
  }, [name, scope, description, parameters, formula, model, editing]);

  const isValid =
    !check.name &&
    check.parameters.every((error) => !error) &&
    check.formula.length === 0;

  const handleSave = () => {
    if (!isValid) {
      return;
    }
    const error = onSave(toInput());
    setSaveError(error ?? "");
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    e.stopPropagation();
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      handleSave();
    }
  };

  const addParameter = () => {
    setParameters([
      ...parameters,
      { key: nextKey, name: "", description: "", optional: false },
    ]);
    setNextKey(nextKey + 1);
  };

  const updateParameter = (index: number, patch: Partial<ParameterRow>) => {
    setParameters(
      parameters.map((p, i) => (i === index ? { ...p, ...patch } : p)),
    );
  };

  const removeParameter = (index: number) => {
    setParameters(parameters.filter((_, i) => i !== index));
  };

  const usageCount = editing
    ? model.namedFunctionUsageCount(editing.name, editing.scope ?? null)
    : null;
  const formulaErrors = [...check.formula, ...(saveError ? [saveError] : [])];
  const formulaHelper =
    formulaErrors.length > 0
      ? formulaErrors.join(" ")
      : check.warnings.length > 0
        ? check.warnings.join(" ")
        : undefined;
  const isSelected = (value: string) => scopeName === value;

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: container captures Cmd/Ctrl+Enter shortcut bubbling from child inputs
    <div className="ic-edit-range-container" onKeyDown={handleKeyDown}>
      <div className="ic-edit-range-content-area">
        <div
          className={
            hidden
              ? "ic-edit-range-header-box ic-edit-range-header-box--hidden"
              : "ic-edit-range-header-box"
          }
        >
          <div className="ic-edit-range-header-icon">
            {hidden ? <EyeOff /> : <SquareFunction />}
          </div>
          <span className="ic-edit-range-header-box-text">
            {name || t("named_functions.new_named_function")}
          </span>
        </div>
        <div className="ic-edit-range-styled-box">
          <Input
            id={nameId}
            autoFocus
            type="text"
            label={t("named_functions.function_name")}
            placeholder={t("named_functions.enter_function_name")}
            value={name}
            error={!!check.name}
            helperText={check.name ?? undefined}
            // Names are upper-cased as typed, but for the `_xludf.` prefix of
            // a name that is a built-in function in this language: it is
            // written like that in a formula
            onChange={(e) =>
              setName(
                e.target.value.toUpperCase().replace(/^_XLUDF\./, "_xludf."),
              )
            }
            onKeyDown={handleKeyDown}
          />
          <Select
            label={t("name_manager_dialog.scope_label")}
            helperText={t("named_functions.scope_helper")}
            value={scopeName}
            onChange={(value) =>
              setScopeSheetId(
                worksheets.find((s) => s.name === value)?.sheet_id,
              )
            }
            options={[
              {
                value: "[Global]",
                label: (
                  <>
                    <span
                      className={
                        isSelected("[Global]")
                          ? "ic-edit-range-menu-span ic-edit-range-menu-span--selected"
                          : "ic-edit-range-menu-span"
                      }
                    >
                      {t("name_manager_dialog.workbook")}
                    </span>
                    <span className="ic-edit-range-menu-span-grey">{` ${t(
                      "name_manager_dialog.global",
                    )}`}</span>
                  </>
                ),
                triggerLabel: (
                  <>
                    <span className="ic-edit-range-menu-span">
                      {t("name_manager_dialog.workbook")}
                    </span>
                    <span className="ic-edit-range-menu-span-grey">{` ${t(
                      "name_manager_dialog.global",
                    )}`}</span>
                  </>
                ),
              },
              ...worksheets.map((option) => ({
                value: option.name,
                label: (
                  <span
                    className={
                      isSelected(option.name)
                        ? "ic-edit-range-menu-span ic-edit-range-menu-span--selected"
                        : "ic-edit-range-menu-span"
                    }
                  >
                    {option.name}
                  </span>
                ),
                triggerLabel: <span>{option.name}</span>,
              })),
            ]}
          />
          <Textarea
            label={t("named_functions.description_label")}
            placeholder={t("named_functions.enter_description")}
            helperText={t("named_functions.description_helper")}
            value={description}
            maxLength={MAX_DESCRIPTION_LENGTH}
            onChange={(e) => setDescription(e.target.value)}
            onKeyDown={handleKeyDown}
          />
        </div>
        <div className="ic-edit-range-styled-box ic-named-functions-arguments">
          <div className="ic-named-functions-arguments-header">
            <div className="ic-edit-range-section-title">
              {t("named_functions.arguments")}
            </div>
            <Tooltip title={t("named_functions.add_argument")}>
              <IconButton
                icon={<Plus />}
                onClick={addParameter}
                aria-label={t("named_functions.add_argument")}
              />
            </Tooltip>
          </div>
          {parameters.length > 0 ? (
            <div className="ic-named-functions-arguments-columns">
              <span>{t("named_functions.argument_name")}</span>
              <span>{t("named_functions.argument_description")}</span>
              <span>{t("named_functions.argument_optional")}</span>
            </div>
          ) : null}
          {parameters.map((parameter, index) => {
            const error = check.parameters[index] ?? null;
            const forcedOptional = isForcedOptional(index);
            const optionalCheckbox = (
              <Checkbox
                checked={parameter.optional || forcedOptional}
                disabled={forcedOptional}
                aria-label={t("named_functions.argument_optional")}
                onChange={(optional) => updateParameter(index, { optional })}
              />
            );
            return (
              <div
                key={parameter.key}
                className="ic-named-functions-argument-row"
              >
                <Input
                  type="text"
                  size="sm"
                  aria-label={t("named_functions.argument_name")}
                  placeholder={t("named_functions.argument_name_placeholder")}
                  value={parameter.name}
                  error={!!error}
                  helperText={error ?? undefined}
                  onChange={(e) =>
                    updateParameter(index, { name: e.target.value })
                  }
                  onKeyDown={handleKeyDown}
                />
                <Input
                  type="text"
                  size="sm"
                  aria-label={t("named_functions.argument_description")}
                  placeholder={t(
                    "named_functions.argument_description_placeholder",
                  )}
                  value={parameter.description}
                  maxLength={MAX_DESCRIPTION_LENGTH}
                  onChange={(e) =>
                    updateParameter(index, { description: e.target.value })
                  }
                  onKeyDown={handleKeyDown}
                />
                <div className="ic-named-functions-argument-optional">
                  {forcedOptional ? (
                    <Tooltip
                      title={t("named_functions.argument_optional_forced")}
                    >
                      {optionalCheckbox}
                    </Tooltip>
                  ) : (
                    optionalCheckbox
                  )}
                </div>
                <Tooltip title={t("named_functions.remove_argument")}>
                  <IconButton
                    icon={<Minus />}
                    onClick={() => removeParameter(index)}
                    aria-label={t("named_functions.remove_argument")}
                  />
                </Tooltip>
              </div>
            );
          })}
        </div>
        <div className="ic-edit-range-styled-box ic-named-functions-formula-section">
          <Textarea
            id={formulaId}
            label={t("named_functions.formula")}
            placeholder={t("named_functions.enter_formula")}
            value={formula}
            rows={3}
            error={formulaErrors.length > 0}
            helperText={formulaHelper}
            onChange={(e) => {
              setFormula(e.target.value);
              setSaveError("");
            }}
            onKeyDown={handleKeyDown}
          />
        </div>
        {showAdvanced ? (
          <div className="ic-edit-range-styled-box ic-edit-range-advanced-section">
            <div className="ic-edit-range-section-title">
              {t("name_manager_dialog.advanced")}
            </div>
            <div className="ic-edit-range-hidden-field">
              <Checkbox
                checked={hidden}
                onChange={setHidden}
                label={t("name_manager_dialog.hide_name")}
              />
              <p className="ic-edit-range-hidden-helper">
                {t("name_manager_dialog.hide_name_helper")}
              </p>
            </div>
          </div>
        ) : null}
      </div>
      <div className="ic-edit-range-footer">
        {usageCount !== null ? (
          <span className="ic-named-functions-usage">
            <Info />
            {t("named_functions.used_in_cells", { count: usageCount })}
          </span>
        ) : (
          <Button variant="secondary" onClick={onCancel}>
            {t("named_functions.cancel")}
          </Button>
        )}
        <Button startIcon={<Check />} disabled={!isValid} onClick={handleSave}>
          {t("named_functions.apply")}
        </Button>
      </div>
    </div>
  );
};

export default EditNamedFunction;
