import type { DefinedName, Model } from "@ironcalc/wasm";
import { MenuItem, Paper, Select, styled } from "@mui/material";
import { Check, MousePointerClick, Tag } from "lucide-react";
import { useEffect, useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import { Input } from "../../Input/Input";
import { getFullRangeToString } from "../../util";
import "./edit-name-range.css";

export interface SaveError {
  nameError: string;
  formulaError: string;
}

interface EditNamedRangeProps {
  name: string;
  scope: string;
  formula: string;
  model: Model;
  onSave: (name: string, scope: string, formula: string) => SaveError;
  onCancel: () => void;
  editingDefinedName: DefinedName | null;
}

// HACK: We are using the text structure of the server error
// to add an error here. This is wrong for several reasons:
// 1. There is no i18n
// 2. Server error messages could change with no warning
export function formatOnSaveError(error: string): SaveError {
  if (error.startsWith("Name: ")) {
    return { formulaError: "", nameError: error.slice(6) };
  } else if (error.startsWith("Formula: ")) {
    return { formulaError: error.slice(9), nameError: "" };
  } else if (error.startsWith("Scope: ")) {
    return { formulaError: "", nameError: error.slice(7) };
  }
  // Fallback for other errors
  return { formulaError: error, nameError: "" };
}

const EditNamedRange = ({
  name: initialName,
  scope: initialScope,
  formula: initialFormula,
  onSave,
  onCancel,
  editingDefinedName,
  model,
}: EditNamedRangeProps) => {
  const { t } = useTranslation();
  const getDefaultName = () => {
    if (initialName) return initialName;
    const rangePrefix = t("name_manager_dialog.default_range_prefix");
    let counter = 1;
    let defaultName = `${rangePrefix}${counter}`;
    const worksheets = model.getWorksheetsProperties();
    const scopeIndex = worksheets.findIndex((s) => s.name === initialScope);
    const newScope = scopeIndex >= 0 ? scopeIndex : undefined;
    const definedNameList = model.getDefinedNameList();

    while (
      definedNameList.some(
        (dn) => dn.name === defaultName && dn.scope === newScope,
      )
    ) {
      counter++;
      defaultName = `${rangePrefix}${counter}`;
    }
    return defaultName;
  };

  const [name, setName] = useState(getDefaultName());
  const [scope, setScope] = useState(initialScope);
  const [formula, setFormula] = useState(initialFormula);
  const [nameError, setNameError] = useState<string>("");
  const [formulaError, setFormulaError] = useState<string>("");

  const isSelected = (value: string) => scope === value;

  const nameId = useId();
  const scopeId = useId();
  const formulaId = useId();

  // Validate name (format and duplicates)
  useEffect(() => {
    const worksheets = model.getWorksheetsProperties();
    const scopeIndex = worksheets.findIndex((s) => s.name === scope);
    const newScope = scopeIndex >= 0 ? scopeIndex : null;
    try {
      model.isValidDefinedName(name, newScope, formula);
    } catch (e) {
      const message = (e as Error).message;
      if (editingDefinedName && message.includes("already exists")) {
        // Allow the same name if it's the one being edited
        setNameError("");
        setFormulaError("");
        return;
      }
      const { nameError, formulaError } = formatOnSaveError(message);
      setNameError(nameError);
      setFormulaError(formulaError);
      return;
    }
    setNameError("");
    setFormulaError("");
  }, [name, scope, formula, model, editingDefinedName]);

  const hasAnyError = nameError !== "" || formulaError !== "";

  return (
    <div className="ic-edit-range-container">
      <div className="ic-edit-range-content-area">
        <div className="ic-edit-range-header-box">
          <div className="ic-edit-range-header-icon">
            <Tag />
          </div>
          <span className="ic-edit-range-header-box-text">
            {name || t("name_manager_dialog.new_named_range")}
          </span>
        </div>
        <div className="ic-edit-range-styled-box">
          <Input
            id={nameId}
            autoFocus
            type="text"
            label={t("name_manager_dialog.range_name")}
            placeholder={t("name_manager_dialog.enter_range_name")}
            value={name}
            error={!!nameError}
            helperText={nameError}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => e.stopPropagation()}
            onClick={(e) => e.stopPropagation()}
          />
          <div className="ic-edit-range-field-wrapper">
            <label className="ic-edit-range-label" htmlFor={scopeId}>
              {t("name_manager_dialog.scope_label")}
            </label>
            <div className="ic-edit-range-form-control">
              <StyledSelect
                id={scopeId}
                value={scope}
                onChange={(event) => {
                  setScope(event.target.value as string);
                }}
                renderValue={(value: unknown) => {
                  const stringValue = value as string;
                  return stringValue === "[Global]" ? (
                    <>
                      <span className="ic-edit-range-menu-span">
                        {t("name_manager_dialog.workbook")}
                      </span>
                      <span className="ic-edit-range-menu-span-grey">{` ${t(
                        "name_manager_dialog.global",
                      )}`}</span>
                    </>
                  ) : (
                    stringValue
                  );
                }}
                MenuProps={{
                  PaperProps: {
                    component: StyledMenuPaper,
                  },
                  anchorOrigin: {
                    vertical: "bottom",
                    horizontal: "center",
                  },
                  transformOrigin: {
                    vertical: "top",
                    horizontal: "center",
                  },
                  marginThreshold: 0,
                }}
              >
                <StyledMenuItem value={"[Global]"}>
                  {isSelected("[Global]") ? (
                    <Check className="ic-edit-range-check-icon" />
                  ) : (
                    <div className="ic-edit-range-icon-placeholder" />
                  )}
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
                </StyledMenuItem>
                {model.getWorksheetsProperties().map((option) => (
                  <StyledMenuItem key={option.name} value={option.name}>
                    {isSelected(option.name) ? (
                      <Check className="ic-edit-range-check-icon" />
                    ) : (
                      <div className="ic-edit-range-icon-placeholder" />
                    )}
                    <span
                      className={
                        isSelected(option.name)
                          ? "ic-edit-range-menu-span ic-edit-range-menu-span--selected"
                          : "ic-edit-range-menu-span"
                      }
                    >
                      {option.name}
                    </span>
                  </StyledMenuItem>
                ))}
              </StyledSelect>
              <span className="ic-edit-range-helper-text">
                {t("name_manager_dialog.scope_helper")}
              </span>
            </div>
          </div>
          <div className="ic-edit-range-field-wrapper">
            <div className="ic-edit-range-line-wrapper">
              <label className="ic-edit-range-label" htmlFor={formulaId}>
                {t("name_manager_dialog.refers_to")}
              </label>
              <MousePointerClick
                size={16}
                onClick={() => {
                  const worksheetNames = model
                    .getWorksheetsProperties()
                    .map((s) => s.name);
                  const selectedView = model.getSelectedView();
                  const formula = getFullRangeToString(
                    selectedView,
                    worksheetNames,
                  );
                  setFormula(formula);
                }}
              />
            </div>
            <div className="ic-edit-range-form-control">
              <textarea
                id={formulaId}
                className={`ic-edit-range-textarea ${
                  formulaError ? "ic-edit-range-textarea--error" : ""
                }`}
                placeholder={t("name_manager_dialog.enter_formula")}
                rows={3}
                value={formula}
                onChange={(e) => {
                  setFormula(e.target.value);
                  setFormulaError("");
                }}
                onKeyDown={(e) => e.stopPropagation()}
                onClick={(e) => e.stopPropagation()}
                aria-invalid={formulaError ? "true" : "false"}
              />
              {formulaError && (
                <span className="ic-edit-range-helper-text ic-edit-range-error-text">
                  {formulaError}
                </span>
              )}
            </div>
          </div>
        </div>
      </div>
      <div className="ic-edit-range-footer">
        <Button variant="secondary" onClick={onCancel}>
          {t("name_manager_dialog.cancel")}
        </Button>
        <Button
          startIcon={<Check />}
          disabled={hasAnyError}
          onClick={() => {
            const error = onSave(name.trim(), scope, formula);
            if (error.nameError) {
              setNameError(error.nameError);
            }
            if (error.formulaError) {
              setFormulaError(error.formulaError);
            }
          }}
        >
          {t("name_manager_dialog.apply")}
        </Button>
      </div>
    </div>
  );
};

const StyledSelect = styled(Select)(() => ({
  fontFamily: "Inter",
  fontSize: "12px",
  "& .MuiSelect-select": {
    padding: "8px",
  },
}));

const StyledMenuPaper = styled(Paper)(() => ({
  padding: 4,
  marginTop: "4px",
  "&.MuiPaper-root": {
    borderRadius: "8px",
  },
  "& .MuiList-padding": {
    padding: 0,
  },
  "& .MuiList-root": {
    padding: 0,
  },
}));

const StyledMenuItem = styled(MenuItem)(({ theme }) => ({
  padding: 8,
  borderRadius: 4,
  display: "flex",
  alignItems: "center",
  "&.Mui-selected": {
    backgroundColor: "transparent",
    "&:hover": {
      backgroundColor: theme.palette.grey[50],
    },
  },
  "&:hover": {
    backgroundColor: theme.palette.grey[50],
  },
}));

export default EditNamedRange;
