import type { DefinedName, Model } from "@ironcalc/wasm";
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
import NamedRange from "./NamedRange";

interface NameManagerDialogProperties {
  onClose: () => void;
  open: boolean;
  model: Model;
}

function NameManagerDialog({
  onClose,
  open,
  model,
}: NameManagerDialogProperties) {
  const [definedNamesLocal, setDefinedNamesLocal] = useState<DefinedName[]>();
  const [showNewName, setShowNewName] = useState(false);
  const [showOptions, setShowOptions] = useState(true);

  useEffect(() => {
    // render definedNames from model
    if (open) {
      const definedNamesModel = model.getDefinedNameList();
      setDefinedNamesLocal(definedNamesModel);
    }
    setShowNewName(false);
    setShowOptions(true);
  }, [open, model]);

  const handleNewName = () => {
    setShowNewName(true);
    setShowOptions(false);
  };

  const handleDelete = () => {
    // re-render modal
    setDefinedNamesLocal(model.getDefinedNameList());
  };

  const formatFormula = (): string => {
    const worksheets = model.getWorksheetsProperties();
    const selectedView = model.getSelectedView();

    return getFullRangeToString(selectedView, worksheets);
  };

  const toggleOptions = () => {
    setShowOptions(!showOptions);
  };

  const toggleShowNewName = () => {
    setShowNewName(false);
  };

  return (
    <StyledDialog open={open} onClose={onClose} maxWidth={false} scroll="paper">
      <StyledDialogTitle>
        {t("name_manager_dialog.title")}
        <IconButton onClick={() => onClose()}>
          <X size={16} />
        </IconButton>
      </StyledDialogTitle>
      <StyledDialogContent dividers>
        <StyledRangesHeader>
          <Box width="171px">{t("name_manager_dialog.name")}</Box>
          <Box width="171px">{t("name_manager_dialog.range")}</Box>
          <Box width="171px">{t("name_manager_dialog.scope")}</Box>
        </StyledRangesHeader>
        {definedNamesLocal?.map((definedName) => (
          <NamedRange
            model={model}
            worksheets={model.getWorksheetsProperties()}
            name={definedName.name}
            scope={definedName.scope}
            formula={definedName.formula}
            key={definedName.name}
            showOptions={showOptions}
            toggleOptions={toggleOptions}
            onDelete={handleDelete}
          />
        ))}
        {showNewName && (
          <NamedRange
            model={model}
            worksheets={model.getWorksheetsProperties()}
            formula={formatFormula()}
            showOptions={showOptions}
            toggleOptions={toggleOptions}
            toggleShowNewName={toggleShowNewName}
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
          onClick={handleNewName}
          variant="contained"
          disableElevation
          sx={{ textTransform: "none" }}
          startIcon={<Plus size={16} />}
          disabled={!showOptions} // disable when editing
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
    minWidth: "620px",
  },
}));

const StyledDialogTitle = styled(DialogTitle)`
padding: 12px 20px;
font-size: 14px;
font-weight: 600;
display: flex;
align-items: center;
justify-content: space-between;
`;

const StyledDialogContent = styled(DialogContent)`
display: flex;
flex-direction: column;
gap: 12px;
padding: 20px 12px 20px 20px;
`;

const StyledRangesHeader = styled(Stack)(({ theme }) => ({
  flexDirection: "row",
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
