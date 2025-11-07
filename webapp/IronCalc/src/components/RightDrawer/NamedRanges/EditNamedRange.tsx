import type { WorksheetProperties } from "@ironcalc/wasm";
import { Box, IconButton, MenuItem, TextField, styled } from "@mui/material";
import { t } from "i18next";
import { Check, X } from "lucide-react";
import { useState } from "react";
import type React from "react";
import { theme } from "../../../theme";

interface EditNamedRangeProps {
  worksheets: WorksheetProperties[];
  name: string;
  scope: string;
  formula: string;
  onSave: (name: string, scope: string, formula: string) => string | undefined;
  onCancel: () => void;
}

const EditNamedRange: React.FC<EditNamedRangeProps> = ({
  worksheets,
  name: initialName,
  scope: initialScope,
  formula: initialFormula,
  onSave,
  onCancel,
}) => {
  const [name, setName] = useState(initialName);
  const [scope, setScope] = useState(initialScope);
  const [formula, setFormula] = useState(initialFormula);
  const [formulaError, setFormulaError] = useState(false);

  return (
    <Container>
      <ContentArea>
        <StyledBox>
          <FieldWrapper>
            <StyledLabel htmlFor="name">Range name</StyledLabel>
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
          </FieldWrapper>
          <FieldWrapper>
            <StyledLabel htmlFor="scope">Scope</StyledLabel>
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
          </FieldWrapper>
          <FieldWrapper>
            <StyledLabel htmlFor="formula">Refers to</StyledLabel>
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
          </FieldWrapper>
        </StyledBox>
      </ContentArea>
      <Footer>
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
      </Footer>
    </Container>
  );
};

const Container = styled("div")({
  height: "100%",
  display: "flex",
  flexDirection: "column",
});

const ContentArea = styled("div")({
  flex: 1,
  overflow: "auto",
});

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
  flex-direction: column;
  align-items: center;
  gap: 12px;
  width: auto;
  padding: 12px;

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

const Footer = styled("div")`
  padding: 8px;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  font-size: 12px;
  color: ${theme.palette.grey["600"]};
  border-top: 1px solid ${theme.palette.grey["300"]};
`;

const IconsWrapper = styled(Box)({
  display: "flex",
});

const FieldWrapper = styled(Box)`
  display: flex;
  flex-direction: column;
  width: 100%;
  gap: 4px;
`;

const StyledLabel = styled("label")`
  font-size: 12px;
  font-family: "Inter";
  font-weight: 500;
  color: ${theme.palette.text.primary};
  display: block;
`;

export default EditNamedRange;
