import type { DefinedName, WorksheetProperties } from "@ironcalc/wasm";
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
import NamedRangeActive from "./NamedRangeActive";
import NamedRangeInactive from "./NamedRangeInactive";

export interface NameManagerProperties {
  newDefinedName: (
    name: string,
    scope: number | undefined,
    formula: string,
  ) => void;
  updateDefinedName: (
    name: string,
    scope: number | undefined,
    newName: string,
    newScope: number | undefined,
    newFormula: string,
  ) => void;
  deleteDefinedName: (name: string, scope: number | undefined) => void;
  selectedArea: () => string;
  worksheets: WorksheetProperties[];
  definedNameList: DefinedName[];
}

interface NameManagerDialogProperties {
  open: boolean;
  onClose: () => void;
  model: NameManagerProperties;
}

function NameManagerDialog(properties: NameManagerDialogProperties) {
  const { open, model, onClose } = properties;
  const {
    newDefinedName,
    updateDefinedName,
    deleteDefinedName,
    selectedArea,
    worksheets,
    definedNameList,
  } = model;
  // If editingNameIndex is -1, then we are adding a new name
  // If editingNameIndex is -2, then we are not editing any name
  // If editingNameIndex is a positive number, then we are editing that index
  const [editingNameIndex, setEditingNameIndex] = useState(-2);

  useEffect(() => {
    if (open) {
      setEditingNameIndex(-2);
    }
  }, [open]);

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
                    newFormula,
                  ): string | undefined => {
                    const scope_index = worksheets.findIndex(
                      (s) => s.name === newScope,
                    );
                    const scope = scope_index > 0 ? scope_index : undefined;
                    try {
                      updateDefinedName(
                        definedName.name,
                        definedName.scope,
                        newName,
                        scope,
                        newFormula,
                      );
                      setEditingNameIndex(-2);
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
                showOptions={editingNameIndex === -2}
                onEdit={() => setEditingNameIndex(index)}
                onDelete={() => {
                  deleteDefinedName(definedName.name, definedName.scope);
                }}
              />
            );
          })}
        </NameListWrapper>
        {editingNameIndex === -1 && (
          <NamedRangeActive
            worksheets={worksheets}
            name={""}
            formula={selectedArea()}
            scope={"[global]"}
            onSave={(name, scope, formula): string | undefined => {
              const scope_index = worksheets.findIndex((s) => s.name === scope);
              const scope_value = scope_index > 0 ? scope_index : undefined;
              try {
                newDefinedName(name, scope_value, formula);
                setEditingNameIndex(-2);
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
          <UploadFooterLink
            href="https://docs.ironcalc.com/web-application/name-manager.html"
            target="_blank"
            rel="noopener noreferrer"
          >
            {t("name_manager_dialog.help")}
          </UploadFooterLink>
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

const UploadFooterLink = styled("a")`
  font-size: 12px;
  font-weight: 400;
  font-family: "Inter";
  color: #757575;
  text-decoration: none;
  &:hover {
    text-decoration: underline;
  }
`;

export default NameManagerDialog;
