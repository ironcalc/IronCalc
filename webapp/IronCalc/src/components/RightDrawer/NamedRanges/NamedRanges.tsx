import type { DefinedName, Model } from "@ironcalc/wasm";
import {
  ArrowLeft,
  PackageOpen,
  PencilLine,
  Plus,
  Trash2,
  X,
} from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import { IconButton } from "../../Button/IconButton";
import { parseRangeInSheet } from "../../Editor/util";
import { Tooltip } from "../../Tooltip/Tooltip";
import EditNamedRange, {
  formatOnSaveError,
  type SaveError,
} from "./EditNamedRange";
import "./named-ranges.css";

const normalizeRangeString = (range: string): string => {
  return range.trim().replace(/['"]/g, "");
};

interface NamedRangesProps {
  onClose: () => void;
  model: Model;
  getSelectedArea: () => string;
  onUpdate: () => void;
}

const NamedRanges = ({
  onClose,
  getSelectedArea,
  model,
  onUpdate,
}: NamedRangesProps) => {
  const [editingDefinedName, setEditingDefinedName] =
    useState<DefinedName | null>(null);
  const [isCreatingNew, setIsCreatingNew] = useState(false);
  const { t } = useTranslation();

  const handleListItemClick = (definedName: DefinedName) => {
    setEditingDefinedName(definedName);
    setIsCreatingNew(false);
  };

  const handleNewClick = () => {
    setIsCreatingNew(true);
    setEditingDefinedName(null);
  };

  const handleCancel = () => {
    setEditingDefinedName(null);
    setIsCreatingNew(false);
  };

  const handleSave = (
    name: string,
    scope: string,
    formula: string,
  ): SaveError => {
    const worksheets = model.getWorksheetsProperties();
    if (isCreatingNew) {
      const scope_index = worksheets.findIndex((s) => s.name === scope);
      const newScope = scope_index >= 0 ? scope_index : null;
      try {
        model.newDefinedName(name, newScope, formula);
        setIsCreatingNew(false);
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
      if (!editingDefinedName)
        return {
          formulaError: "",
          nameError: "",
        };

      const scope_index = worksheets.findIndex((s) => s.name === scope);
      const newScope = scope_index >= 0 ? scope_index : null;
      try {
        model.updateDefinedName(
          editingDefinedName.name,
          editingDefinedName.scope ?? null,
          name,
          newScope,
          formula,
        );
        setEditingDefinedName(null);
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

  // Show edit view if a named range is being edited or created
  if (editingDefinedName || isCreatingNew) {
    let name = "";
    let scopeName = "[Global]";
    let formula = "";

    if (editingDefinedName) {
      name = editingDefinedName.name;
      const worksheets = model.getWorksheetsProperties();
      scopeName =
        editingDefinedName.scope != null
          ? worksheets[editingDefinedName.scope]?.name || "[unknown]"
          : "[Global]";
      formula = editingDefinedName.formula;
    } else if (isCreatingNew) {
      formula = getSelectedArea();
    }

    const headerTitle = isCreatingNew
      ? t("name_manager_dialog.add_new_range")
      : t("name_manager_dialog.edit_range");

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
          <EditNamedRange
            name={name}
            scope={scopeName}
            formula={formula}
            onSave={handleSave}
            onCancel={handleCancel}
            editingDefinedName={editingDefinedName}
            model={model}
          />
        </div>
      </div>
    );
  }

  const currentSelectedArea = getSelectedArea();
  const definedNameList = model.getDefinedNameList();
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
        <div className="ic-named-ranges-header-title">
          {t("name_manager_dialog.title")}
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
        {definedNameList.length === 0 ? (
          <div className="ic-named-ranges-empty-state-message">
            <div className="ic-named-ranges-icon-wrapper">
              <PackageOpen />
            </div>
            {t("name_manager_dialog.empty_message1")}
            <br />
            {t("name_manager_dialog.empty_message2")}
          </div>
        ) : (
          <div className="ic-named-ranges-list-container">
            {definedNameList.map((definedName) => {
              const worksheets = model.getWorksheetsProperties();
              const scopeName =
                definedName.scope != null
                  ? worksheets[definedName.scope]?.name || "[Unknown]"
                  : "[Global]";
              const isSelected =
                currentSelectedArea !== null &&
                normalizeRangeString(definedName.formula) ===
                  normalizeRangeString(currentSelectedArea);
              return (
                // biome-ignore lint/a11y/noStaticElementInteractions: FIXME
                <div
                  className={`ic-named-ranges-list-item ${isSelected ? "selected" : ""}`}
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
                      {definedName.name}
                    </div>
                    <div className="ic-named-ranges-scope-text">
                      {scopeName}
                    </div>
                    <div className="ic-named-ranges-formula-text">
                      {definedName.formula}
                    </div>
                  </div>

                  <div className="ic-named-ranges-icons-wrapper">
                    <Tooltip title={t("name_manager_dialog.edit")}>
                      <IconButton
                        icon={<PencilLine />}
                        onClick={(e) => {
                          e.stopPropagation();
                          handleListItemClick(definedName);
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
          </div>
        )}
      </div>
      <div className="ic-named-ranges-footer">
        <Button startIcon={<Plus />} onClick={handleNewClick}>
          {t("name_manager_dialog.new")}
        </Button>
      </div>
    </div>
  );
};

export default NamedRanges;
