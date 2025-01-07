import type { WorksheetProperties } from "@ironcalc/wasm";
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
import { theme } from "../../theme";

interface NamedRangeProperties {
  worksheets: WorksheetProperties[];
  name: string;
  scope: string;
  formula: string;
  onSave: (name: string, scope: string, formula: string) => string | undefined;
  onCancel: () => void;
}

function NamedRangeActive(properties: NamedRangeProperties) {
  const { worksheets, onSave, onCancel } = properties;
  const [name, setName] = useState(properties.name);
  const [scope, setScope] = useState(properties.scope);
  const [formula, setFormula] = useState(properties.formula);

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
          error={formulaError}
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
          error={formulaError}
          value={scope}
          onChange={(event) => {
            setScope(event.target.value);
          }}
        >
          <MenuItem value={"[global]"}>
            <MenuSpan>{t("name_manager_dialog.workbook")}</MenuSpan>
            <MenuSpanGrey>{` ${t("name_manager_dialog.global")}`}</MenuSpanGrey>
          </MenuItem>
          {worksheets.map((option) => (
            <MenuItem key={option.name} value={option.name}>
              <MenuSpan>{option.name}</MenuSpan>
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
          <StyledIconButton
            onClick={() => {
              const error = onSave(name, scope, formula);
              if (error) {
                setFormulaError(true);
              }
            }}
            title={t("name_manager_dialog.apply")}
          >
            <StyledCheck size={16} />
          </StyledIconButton>
          <StyledIconButton
            onClick={onCancel}
            title={t("name_manager_dialog.discard")}
          >
            <X size={16} />
          </StyledIconButton>
        </IconsWrapper>
      </StyledBox>
      <Divider />
    </>
  );
}

const MenuSpan = styled("span")`
  font-size: 12px;
  font-family: "Inter";
`;

const MenuSpanGrey = styled("span")`
  white-space: pre;
  font-size: 12px;
  font-family: "Inter";
  color: ${theme.palette.grey[400]};
`;

const StyledBox = styled(Box)`
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 12px;
  width: auto;
  padding: 10px 20px 10px 12px;
  box-shadow: 0 -1px 0 ${theme.palette.grey[300]};

  @media (max-width: 600px) {
    padding: 12px;
  }
`;

const StyledTextField = styled(TextField)(() => ({
  "& .MuiInputBase-root": {
    height: "36px",
    width: "100%",
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
  borderRadius: "8px",
  "&:hover": {
    backgroundColor: theme.palette.grey["50"],
  },
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
