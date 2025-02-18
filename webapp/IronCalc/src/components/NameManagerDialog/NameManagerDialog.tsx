import type { DefinedName, WorksheetProperties } from "@ironcalc/wasm";
import {
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Stack,
  styled,
} from "@mui/material";
import { t } from "i18next";
import { BookOpen, PackageOpen, Plus, X } from "lucide-react";
import { useEffect, useState } from "react";
import { theme } from "../../theme";
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
  const handleClose = () => {
    properties.onClose();
  };

  return (
    <StyledDialog open={open} onClose={onClose} maxWidth={false} scroll="paper">
      <StyledDialogTitle>
        {t("name_manager_dialog.title")}
        <Cross
          onClick={handleClose}
          title={t("name_manager_dialog.close")}
          tabIndex={0}
          onKeyDown={(event) => event.key === "Enter" && properties.onClose()}
        >
          <X />
        </Cross>
      </StyledDialogTitle>
      <StyledDialogContent>
        {(definedNameList.length > 0 || editingNameIndex !== -2) && (
          <StyledRangesHeader>
            <StyledBox>{t("name_manager_dialog.name")}</StyledBox>
            <StyledBox>{t("name_manager_dialog.range")}</StyledBox>
            <StyledBox>{t("name_manager_dialog.scope")}</StyledBox>
          </StyledRangesHeader>
        )}
        {definedNameList.length === 0 && editingNameIndex === -2 ? (
          <EmptyStateMessage>
            <IconWrapper>
              <PackageOpen />
            </IconWrapper>
            {t("name_manager_dialog.empty_message1")}
            <br />
            {t("name_manager_dialog.empty_message2")}
          </EmptyStateMessage>
        ) : (
          <NameListWrapper>
            {definedNameList.map((definedName, index) => {
              const scopeName =
                definedName.scope !== undefined
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
                      const scope = scope_index >= 0 ? scope_index : undefined;
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
        )}
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
          sx={{ textTransform: "none", minWidth: "fit-content" }}
          startIcon={<Plus size={16} />}
          disabled={editingNameIndex > -2}
        >
          {t("name_manager_dialog.new")}
        </Button>
      </StyledDialogActions>
    </StyledDialog>
  );
}

const StyledDialog = styled(Dialog)(({ theme }) => ({
  "& .MuiPaper-root": {
    height: "400px",
    minHeight: "200px",
    minWidth: "620px",
    maxWidth: "620px",
    [theme.breakpoints.down("sm")]: {
      minWidth: "90%",
    },
  },
}));

const StyledDialogTitle = styled(DialogTitle)`
  display: flex;
  align-items: center;
  height: 44px;
  font-size: 14px;
  font-weight: 500;
  font-family: Inter;
  padding: 0px 12px;
  justify-content: space-between;
  border-bottom: 1px solid ${theme.palette.grey["300"]};
`;

const Cross = styled("div")`
  &:hover {
    background-color: ${theme.palette.grey["50"]};
  }
  display: flex;
  border-radius: 4px;
  height: 24px;
  width: 24px;
  cursor: pointer;
  align-items: center;
  justify-content: center;
  svg {
    width: 16px;
    height: 16px;
    stroke-width: 1.5;
  }
`;

const NameListWrapper = styled(Stack)`
  overflow-y: auto;
`;

const EmptyStateMessage = styled(Box)`
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: center;
  justify-content: center;
  text-align: center;
  width: 100%;
  height: 100%;
  font-size: 12px;
  color: ${theme.palette.grey["600"]};
  font-family: "Inter";
  z-index: 0;
  margin: auto 0px;
  position: relative;
`;

const IconWrapper = styled("div")`
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 4px;
  background-color: ${theme.palette.grey["100"]};
  color: ${theme.palette.grey["600"]};
  svg {
    width: 16px;
    height: 16px;
    stroke-width: 2;
  }
`;

const StyledBox = styled(Box)`
  display: flex;
  flex-direction: column;
  justify-content: center;
  width: 100%;
  padding-left: 8px;
`;

const StyledDialogContent = styled(DialogContent)`
  display: flex;
  flex-direction: column;
  padding: 0px;
`;

const StyledRangesHeader = styled(Stack)(({ theme }) => ({
  flexDirection: "row",
  minHeight: "32px",
  padding: "0px 96px 0px 12px",
  gap: "12px",
  fontFamily: theme.typography.fontFamily,
  fontSize: "12px",
  fontWeight: "700",
  borderBottom: `1px solid ${theme.palette.info.light}`,
  backgroundColor: theme.palette.grey["50"],
  color: theme.palette.info.main,
}));

const StyledDialogActions = styled(DialogActions)`
  padding: 12px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  color: ${theme.palette.grey["600"]};
  border-top: 1px solid ${theme.palette.grey["300"]};
`;

const UploadFooterLink = styled("a")`
  font-size: 12px;
  font-weight: 400;
  font-family: "Inter";
  color: ${theme.palette.grey["600"]};
  text-decoration: none;
  &:hover {
    text-decoration: underline;
  }
`;

export default NameManagerDialog;
