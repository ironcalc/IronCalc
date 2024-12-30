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
import { Check, PencilLine, Trash2, X } from "lucide-react";
import { useEffect, useState } from "react";

interface NamedRangeProperties {
  model: Model;
  worksheets: WorksheetProperties[];
  name: string;
  scope?: number;
  formula: string;
  onCreate?: () => void;
  onDelete?: () => void;
  toggleShowNewName?: () => void;
  toggleOptions: () => void;
  showOptions?: boolean;
}

function NamedRange(properties: NamedRangeProperties) {
  const {
    model,
    worksheets,
    name,
    scope,
    formula,
    onCreate,
    onDelete,
    toggleShowNewName,
    toggleOptions,
    showOptions,
  } = properties;

  const [newName, setNewName] = useState(name);
  const [newScope, setNewScope] = useState(scope);
  const [newFormula, setNewFormula] = useState(formula);
  const [readOnly, setReadOnly] = useState(true);
  const [showEditDelete, setShowEditDelete] = useState(false);

  // todo: add error messages for validations
  const [nameError, setNameError] = useState(false);
  const [formulaError, setFormulaError] = useState(false);

  useEffect(() => {
    // set state for new name
    const definedNamesModel = model.getDefinedNameList();
    if (!definedNamesModel.find((n) => n.name === newName)) {
      setReadOnly(false);
      setShowEditDelete(true);
    }
  }, [newName, model]);

  const handleCreateUpdate = () => {
    const definedNamesModel = model.getDefinedNameList();

    if (definedNamesModel.find((n) => n.name === name)) {
      // update
      try {
        model.updateDefinedName(name, scope, newName, newScope, newFormula);
      } catch (error) {
        console.log("DefinedName update failed", error);
      }
    } else {
      // create
      try {
        model.newDefinedName(newName, newScope, newFormula);
      } catch (error) {
        console.log("DefinedName save failed", error);
      }
      onCreate?.();
    }
    setShowEditDelete(false);
    toggleOptions();
  };

  const handleCancel = () => {
    setReadOnly(true);
    setShowEditDelete(false);
    toggleOptions();
    setNewName(name);
    setNewScope(scope);
    toggleShowNewName?.(); // if it's newName remove it from modal
  };

  const handleEdit = () => {
    setReadOnly(false);
    setShowEditDelete(true);
    toggleOptions();
  };

  const handleDelete = () => {
    try {
      model.deleteDefinedName(newName, newScope);
    } catch (error) {
      console.log("DefinedName delete failed", error);
    }
    onDelete?.();
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
          InputProps={{ readOnly: readOnly }}
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
          InputProps={{ readOnly: readOnly }}
          value={newScope ?? "global"}
          onChange={(event) => {
            event.target.value === "global"
              ? setNewScope(undefined)
              : setNewScope(+event.target.value);
          }}
        >
          <MenuItem value={"global"}>
            {t("name_manager_dialog.workbook")}
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
          InputProps={{ readOnly: readOnly }}
          error={formulaError}
          value={newFormula}
          onChange={(event) => setNewFormula(event.target.value)}
          onKeyDown={(event) => {
            event.stopPropagation();
          }}
          onClick={(event) => event.stopPropagation()}
        />

        {showEditDelete ? (
          // save cancel
          <>
            <IconButton
              onClick={handleCreateUpdate}
              sx={{ color: (theme) => theme.palette.success.main }}
            >
              <Check size={12} />
            </IconButton>
            <IconButton
              onClick={handleCancel}
              sx={{ color: (theme) => theme.palette.error.main }}
            >
              <X size={12} />
            </IconButton>
          </>
        ) : (
          // edit delete
          <>
            <IconButton onClick={handleEdit} disabled={!showOptions}>
              <PencilLine size={12} />
            </IconButton>
            <StyledIconButton onClick={handleDelete} disabled={!showOptions}>
              <Trash2 size={12} />
            </StyledIconButton>
          </>
        )}
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
    margin: 0,
  },
}));

const StyledIconButton = styled(IconButton)(({ theme }) => ({
  color: theme.palette.error.main,
  "&.Mui-disabled": {
    opacity: 0.6,
    color: theme.palette.error.light,
  },
}));

export default NamedRange;
