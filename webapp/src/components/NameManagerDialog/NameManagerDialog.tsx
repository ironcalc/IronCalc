import type { Model } from "@ironcalc/wasm";
import {
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  IconButton,
  Stack,
  styled,
} from "@mui/material";
import { t } from "i18next";
import { BookOpen, Plus, X } from "lucide-react";
import { useEffect, useState } from "react";
import { getFullRangeToString } from "../util";
import NamedRangeActive from "./NamedRangeActive";
import NamedRangeInactive from "./NamedRangeInactive";

interface NameManagerDialogProperties {
  open: boolean;
  model: Model;
  onClose: () => void;
  onNamesChanged: () => void;
}

function NameManagerDialog(properties: NameManagerDialogProperties) {
  const { open, model, onClose, onNamesChanged } = properties;
  // If editingNameIndex is -1, then we are adding a new name
  // If editingNameIndex is -2, then we are not editing any name
  // If editingNameIndex is a positive number, then we are editing that index
  const [editingNameIndex, setEditingNameIndex] = useState(-2);
  const [showOptions, setShowOptions] = useState(true);
  const worksheets = model.getWorksheetsProperties();
  const definedNameList = model.getDefinedNameList();

  useEffect(() => {
    if (open) {
      setEditingNameIndex(-2);
    }
  }, [open]);

  useEffect(() => {
    if (editingNameIndex !== -2) {
      setShowOptions(false);
    } else {
      setShowOptions(true);
    }
  }, [editingNameIndex]);

  const formatFormula = (): string => {
    const worksheetNames = worksheets.map((s) => s.name);
    const selectedView = model.getSelectedView();

    return getFullRangeToString(selectedView, worksheetNames);
  };

  return (
    <StyledDialog open={open} onClose={onClose} maxWidth={false} scroll="paper">
      <StyledDialogTitle>
        {t("name_manager_dialog.title")}
        <IconButton onClick={onClose}>
          <X size={16} />
        </IconButton>
      </StyledDialogTitle>
      <StyledDialogContent dividers>
        <StyledRangesHeader>
          <StyledBox>{t("name_manager_dialog.name")}</StyledBox>
          <StyledBox>{t("name_manager_dialog.range")}</StyledBox>
          <StyledBox>{t("name_manager_dialog.scope")}</StyledBox>
        </StyledRangesHeader>
        <NameListWrapper>
          {definedNameList.map((definedName, index) => {
            const scopeName = definedName.scope
              ? worksheets[definedName.scope].name
              : "[global]";
            if (index === editingNameIndex) {
              return (
                <NamedRangeActive
                  worksheets={worksheets}
                  name={definedName.name}
                  scope={scopeName}
                  formula={definedName.formula}
                  key={definedName.name + definedName.scope}
                  onSave={(
                    newName,
                    newScope,
                    newFormula
                  ): string | undefined => {
                    const scope_index = worksheets.findIndex(
                      (s) => s.name === newScope
                    );
                    const scope = scope_index > 0 ? scope_index : undefined;
                    try {
                      model.updateDefinedName(
                        definedName.name,
                        definedName.scope,
                        newName,
                        scope,
                        newFormula
                      );
                      setEditingNameIndex(-2);
                      onNamesChanged();
                    } catch (e) {
                      return `${e}`;
                    }
                  }}
                  onCancel={() => setEditingNameIndex(-2)}
                />
              );
            }
            return (
              <NamedRangeInactive
                name={definedName.name}
                scope={scopeName}
                formula={definedName.formula}
                key={definedName.name + definedName.scope}
                showOptions={showOptions}
                onEdit={() => setEditingNameIndex(index)}
                onDelete={() => {
                  model.deleteDefinedName(definedName.name, definedName.scope);
                  onNamesChanged();
                }}
              />
            );
          })}
        </NameListWrapper>
        {editingNameIndex === -1 && (
          <NamedRangeActive
            worksheets={worksheets}
            name={""}
            formula={formatFormula()}
            scope={"[global]"}
            onSave={(name, scope, formula): string | undefined => {
              const scope_index = worksheets.findIndex((s) => s.name === scope);
              const scope_value = scope_index > 0 ? scope_index : undefined;
              try {
                model.newDefinedName(name, scope_value, formula);
                setEditingNameIndex(-2);
                onNamesChanged();
              } catch (e) {
                return `${e}`;
              }
            }}
            onCancel={() => setEditingNameIndex(-2)}
          />
        )}
      </StyledDialogContent>
      <StyledDialogActions>
        <Box display="flex" alignItems="center" gap={"8px"}>
          <BookOpen color="grey" size={16} />
          <span style={{ fontSize: "12px", fontFamily: "Inter" }}>
            {t("name_manager_dialog.help")}
          </span>
        </Box>
        <Button
          onClick={() => setEditingNameIndex(-1)}
          variant="contained"
          disableElevation
          sx={{ textTransform: "none" }}
          startIcon={<Plus size={16} />}
          disabled={editingNameIndex > -2}
        >
          {t("name_manager_dialog.new")}
        </Button>
      </StyledDialogActions>
    </StyledDialog>
  );
}

const StyledDialog = styled(Dialog)(() => ({
  "& .MuiPaper-root": {
    height: "380px",
    minHeight: "200px",
    minWidth: "620px",
  },
}));

const StyledDialogTitle = styled(DialogTitle)`
  padding: 12px 20px;
  height: 20px;
  font-size: 14px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: space-between;
`;

const NameListWrapper = styled(Stack)`
  overflow-y: auto;
  gap: 12px;
`;

const StyledBox = styled(Box)`
  width: 161.67px;
`;

const StyledDialogContent = styled(DialogContent)`
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 20px 12px 20px 20px;
`;

const StyledRangesHeader = styled(Stack)(({ theme }) => ({
  flexDirection: "row",
  padding: "0 8px",
  gap: "12px",
  fontFamily: theme.typography.fontFamily,
  fontSize: "12px",
  fontWeight: "700",
  color: theme.palette.info.main,
}));

const StyledDialogActions = styled(DialogActions)`
  padding: 12px 20px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  color: #757575;
`;

export default NameManagerDialog;
