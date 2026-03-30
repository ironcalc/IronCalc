import type { DefinedName, Model } from "@ironcalc/wasm";
import { styled } from "@mui/material";
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
      <Container>
        <EditHeader>
          <Tooltip title={t("name_manager_dialog.back_to_list")}>
            <IconButton
              icon={<ArrowLeft />}
              onClick={handleCancel}
              aria-label={t("name_manager_dialog.back_to_list")}
            />
          </Tooltip>
          <EditHeaderTitle>{headerTitle}</EditHeaderTitle>
          <Tooltip title={t("right_drawer.close")}>
            <IconButton
              icon={<X />}
              onClick={onClose}
              aria-label={t("right_drawer.close")}
            />
          </Tooltip>
        </EditHeader>
        <Content>
          <EditNamedRange
            name={name}
            scope={scopeName}
            formula={formula}
            onSave={handleSave}
            onCancel={handleCancel}
            editingDefinedName={editingDefinedName}
            model={model}
          />
        </Content>
      </Container>
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
    <Container>
      <Header>
        <HeaderTitle>{t("name_manager_dialog.title")}</HeaderTitle>
        <Tooltip title={t("right_drawer.close")}>
          <IconButton
            icon={<X />}
            onClick={onClose}
            aria-label={t("right_drawer.close")}
          />
        </Tooltip>
      </Header>
      <Content>
        {definedNameList.length === 0 ? (
          <EmptyStateMessage>
            <IconWrapper>
              <PackageOpen />
            </IconWrapper>
            {t("name_manager_dialog.empty_message1")}
            <br />
            {t("name_manager_dialog.empty_message2")}
          </EmptyStateMessage>
        ) : (
          <ListContainer>
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
                <ListItem
                  key={`${definedName.name}-${definedName.scope}`}
                  tabIndex={0}
                  $isSelected={isSelected}
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
                  <ListItemText>
                    <NameText>{definedName.name}</NameText>
                    <ScopeText>{scopeName}</ScopeText>
                    <FormulaText>{definedName.formula}</FormulaText>
                  </ListItemText>

                  <IconsWrapper>
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
                  </IconsWrapper>
                </ListItem>
              );
            })}
          </ListContainer>
        )}
      </Content>
      <Footer>
        <Button startIcon={<Plus />} onClick={handleNewClick}>
          {t("name_manager_dialog.new")}
        </Button>
      </Footer>
    </Container>
  );
};

const Container = styled("div")({
  height: "100%",
  display: "flex",
  flexDirection: "column",
});

const Content = styled("div")(({ theme }) => ({
  flex: 1,
  color: theme.palette.grey[700],
  lineHeight: 1.5,
  overflow: "auto",
}));

const ListContainer = styled("div")({
  display: "flex",
  flexDirection: "column",
});

const ListItem = styled("div")<{ $isSelected: boolean }>(
  ({ theme, $isSelected }) => ({
    display: "flex",
    alignItems: "flex-start",
    justifyContent: "space-between",
    gap: 8,
    padding: "8px 12px",
    cursor: "pointer",
    minHeight: 40,
    boxSizing: "border-box",
    borderBottom: `1px solid ${theme.palette.grey[200]}`,
    paddingLeft: $isSelected ? 20 : 12,
    transition: "all 0.2s ease-in-out",
    borderLeft: $isSelected
      ? `3px solid ${theme.palette.primary.main}`
      : "3px solid transparent",

    "&:hover": {
      backgroundColor: theme.palette.grey[50],
      paddingLeft: 20,
    },
  }),
);

const ListItemText = styled("div")(({ theme }) => ({
  fontSize: 12,
  color: theme.palette.common.black,
  fontFamily: theme.typography.fontFamily,
  flex: 1,
  display: "flex",
  flexDirection: "column",
  alignItems: "flex-start",
  gap: 2,
}));

const ScopeText = styled("span")(({ theme }) => ({
  fontSize: 12,
  color: theme.palette.common.black,
}));

const FormulaText = styled("span")(({ theme }) => ({
  fontSize: 12,
  color: theme.palette.grey[600],
}));

const NameText = styled("span")(({ theme }) => ({
  fontSize: 12,
  color: theme.palette.common.black,
  fontWeight: 600,
  wordBreak: "break-all",
  overflowWrap: "break-word",
}));

const IconsWrapper = styled("div")({
  display: "flex",
  alignItems: "center",
  gap: 2,
});

export const Footer = styled("div")(({ theme }) => ({
  padding: 8,
  display: "flex",
  alignItems: "center",
  justifyContent: "flex-end",
  fontSize: 12,
  color: theme.palette.grey[600],
  borderTop: `1px solid ${theme.palette.grey[300]}`,
  gap: 8,
}));

const Header = styled("div")(({ theme }) => ({
  height: 40,
  display: "flex",
  alignItems: "center",
  justifyContent: "flex-end",
  padding: "0 8px",
  borderBottom: `1px solid ${theme.palette.grey[300]}`,
}));

const HeaderTitle = styled("div")({
  width: "100%",
  fontSize: 12,
});

const EditHeader = styled("div")(({ theme }) => ({
  height: 40,
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  padding: "0 8px",
  gap: 8,
  borderBottom: `1px solid ${theme.palette.grey[300]}`,
}));

const EditHeaderTitle = styled("div")({
  flex: 1,
  fontSize: 12,
  fontWeight: 500,
});

const EmptyStateMessage = styled("div")(({ theme }) => ({
  display: "flex",
  flexDirection: "column",
  gap: 8,
  alignItems: "center",
  justifyContent: "center",
  textAlign: "center",
  width: "100%",
  height: "100%",
  fontSize: 12,
  color: theme.palette.grey[600],
  fontFamily: "Inter",
  zIndex: 0,
  margin: "auto 0px",
  position: "relative",
}));

const IconWrapper = styled("div")(({ theme }) => ({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  width: 36,
  height: 36,
  borderRadius: 4,
  backgroundColor: theme.palette.grey[100],
  color: theme.palette.grey[600],

  "& svg": {
    width: 16,
    height: 16,
    strokeWidth: 2,
  },
}));

export default NamedRanges;
