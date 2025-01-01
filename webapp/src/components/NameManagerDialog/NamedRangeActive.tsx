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
  scope: string;
  formula: string;
  onSave: (name: string, scope: string, formula: string) => void;
  onCancel: () => void;
}

function NamedRangeActive(properties: NamedRangeProperties) {
  const { worksheets, onSave, onCancel } = properties;
  const [name, setName] = useState(properties.name);
  const [scope, setScope] = useState(properties.scope);
  const [formula, setFormula] = useState(properties.formula);

  // TODO: add error messages for validations
  const [nameError, setNameError] = useState(false);
  const [formulaError, setFormulaError] = useState(false);

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
          value={name}
          onChange={(event) => setName(event.target.value)}
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
          value={scope}
          onChange={(event) => {
            setScope(event.target.value);
          }}
        >
          <MenuItem value={"[global]"}>
            {`${t("name_manager_dialog.workbook")} ${t(
              "name_manager_dialog.global",
            )}`}
          </MenuItem>
          {worksheets.map((option) => (
            <MenuItem key={option.name} value={option.name}>
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
          value={formula}
          onChange={(event) => setFormula(event.target.value)}
          onKeyDown={(event) => {
            event.stopPropagation();
          }}
          onClick={(event) => event.stopPropagation()}
        />
        <IconsWrapper>
          <IconButton
            onClick={() => {
              onSave(name, scope, formula);
            }}
          >
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
