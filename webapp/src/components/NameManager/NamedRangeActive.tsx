import type { Model, WorksheetProperties } from "@ironcalc/wasm";
import {
  Box,
  Divider,
  IconButton,
  MenuItem,
  TextField,
  styled,
} from "@mui/material";
import { t } from "i18next";
import { Check, X } from "lucide-react";
import { useState } from "react";

interface NamedRangeProperties {
  model: Model;
  worksheets: WorksheetProperties[];
  name: string;
  scope?: number;
  formula: string;
  onSave: () => void;
  onDelete?: () => void;
  onCancel?: () => void;
}

function NamedRangeActive(properties: NamedRangeProperties) {
  const { model, worksheets, name, scope, formula, onSave, onCancel } =
    properties;
  const [newName, setNewName] = useState(name);
  const [newScope, setNewScope] = useState(scope);
  const [newFormula, setNewFormula] = useState(formula);

  // todo: add error messages for validations
  const [nameError, setNameError] = useState(false);
  const [formulaError, setFormulaError] = useState(false);

  //todo: move logic to NameManagerDialog
  const handleSaveUpdate = () => {
    const definedNamesModel = model.getDefinedNameList();

    if (definedNamesModel.find((n) => n.name === name && n.scope === scope)) {
      try {
        model.updateDefinedName(name, scope, newName, newScope, newFormula);
      } catch (error) {
        console.log("DefinedName update failed", error);
      }
    } else {
      try {
        model.newDefinedName(newName, newScope, newFormula);
      } catch (error) {
        console.log("DefinedName save failed", error);
      }
    }
    onSave();
  };

  return (
    <>
      <StyledBox>
        <StyledTextField
          id="name"
          variant="outlined"
          size="small"
          margin="none"
          fullWidth
          error={nameError}
          value={newName}
          onChange={(event) => setNewName(event.target.value)}
          onKeyDown={(event) => {
            event.stopPropagation();
          }}
          onClick={(event) => event.stopPropagation()}
        />
        <StyledTextField
          id="scope"
          variant="outlined"
          select
          size="small"
          margin="none"
          fullWidth
          value={newScope ?? "global"}
          onChange={(event) => {
            event.target.value === "global"
              ? setNewScope(undefined)
              : setNewScope(+event.target.value);
          }}
        >
          <MenuItem value={"global"}>
            {`${t("name_manager_dialog.workbook")} ${t("name_manager_dialog.global")}`}
          </MenuItem>
          {worksheets.map((option, index) => (
            <MenuItem key={option.name} value={index}>
              {option.name}
            </MenuItem>
          ))}
        </StyledTextField>
        <StyledTextField
          id="formula"
          variant="outlined"
          size="small"
          margin="none"
          fullWidth
          error={formulaError}
          value={newFormula}
          onChange={(event) => setNewFormula(event.target.value)}
          onKeyDown={(event) => {
            event.stopPropagation();
          }}
          onClick={(event) => event.stopPropagation()}
        />
        <IconsWrapper>
          <IconButton onClick={handleSaveUpdate}>
            <StyledCheck size={12} />
          </IconButton>
          <StyledIconButton onClick={onCancel}>
            <X size={12} />
          </StyledIconButton>
        </IconsWrapper>
      </StyledBox>
      <Divider />
    </>
  );
}

const StyledBox = styled(Box)`
display: flex;
gap: 12px;
width: 577px;
`;

const StyledTextField = styled(TextField)(() => ({
  "& .MuiInputBase-root": {
    height: "28px",
    width: "161.67px",
    margin: 0,
    fontFamily: "Inter",
    fontSize: "12px",
  },
  "& .MuiInputBase-input": {
    padding: "8px",
  },
}));

const StyledIconButton = styled(IconButton)(({ theme }) => ({
  color: theme.palette.error.main,
  "&.Mui-disabled": {
    opacity: 0.6,
    color: theme.palette.error.light,
  },
}));

const StyledCheck = styled(Check)(({ theme }) => ({
  color: theme.palette.success.main,
}));

const IconsWrapper = styled(Box)({
  display: "flex",
});

export default NamedRangeActive;
