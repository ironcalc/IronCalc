import type { WorksheetProperties } from "@ironcalc/wasm";
import { Box, MenuItem, TextField, styled } from "@mui/material";
import { t } from "i18next";
import { Check, Tag } from "lucide-react";
import { useState } from "react";
import type React from "react";
import { theme } from "../../../theme";
import { Footer, NewButton } from "./NamedRanges";

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
        <HeaderBox>
          <HeaderIcon>
            <Tag />
          </HeaderIcon>
          <HeaderBoxText>
            {name || t("name_manager_dialog.new_named_range")}
          </HeaderBoxText>
        </HeaderBox>
        <StyledBox>
          <FieldWrapper>
            <StyledLabel htmlFor="name">
              {t("name_manager_dialog.range_name")}
            </StyledLabel>
            <StyledTextField
              autoFocus={true}
              id="name"
              variant="outlined"
              size="small"
              margin="none"
              placeholder={t("name_manager_dialog.enter_range_name")}
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
            <StyledLabel htmlFor="scope">
              {t("name_manager_dialog.scope_label")}
            </StyledLabel>
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
              <MenuItem value={"[Global]"}>
                <MenuSpan>{t("name_manager_dialog.workbook")}</MenuSpan>
                <MenuSpanGrey>{` ${t("name_manager_dialog.global")}`}</MenuSpanGrey>
              </MenuItem>
              {worksheets.map((option) => (
                <MenuItem key={option.name} value={option.name}>
                  <MenuSpan>{option.name}</MenuSpan>
                </MenuItem>
              ))}
            </StyledTextField>
            <StyledHelperText>
              {t("name_manager_dialog.scope_helper")}
            </StyledHelperText>
          </FieldWrapper>
          <FieldWrapper>
            <StyledLabel htmlFor="formula">
              {t("name_manager_dialog.refers_to")}
            </StyledLabel>
            <StyledTextField
              id="formula"
              variant="outlined"
              size="small"
              margin="none"
              fullWidth
              multiline
              rows={3}
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
        <NewButton
          variant="contained"
          color="secondary"
          disableElevation
          onClick={onCancel}
        >
          {t("name_manager_dialog.cancel")}
        </NewButton>
        <NewButton
          variant="contained"
          disableElevation
          startIcon={<Check size={16} />}
          onClick={() => {
            const error = onSave(name, scope, formula);
            if (error) {
              setFormulaError(true);
            }
          }}
        >
          {t("name_manager_dialog.apply")}
        </NewButton>
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

const HeaderBox = styled(Box)`
  font-size: 14px;
  font-family: "Inter";
  font-weight: 600;
  width: auto;
  gap: 8px;
  padding: 24px 12px;
  color: ${theme.palette.text.primary};
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  border-bottom: 1px solid ${theme.palette.grey["200"]};
  `;

const HeaderBoxText = styled("span")`
  max-width: 100%;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  `;

const HeaderIcon = styled(Box)`
  width: 28px;
  height: 28px;
  border-radius: 4px;
  background-color: ${theme.palette.grey["100"]};
  display: flex;
  align-items: center;
  justify-content: center;
  svg {
    width: 16px;
    height: 16px;
    color: ${theme.palette.grey["600"]};
  }
`;

const StyledBox = styled(Box)`
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  width: auto;
  padding: 16px 12px;

  @media (max-width: 600px) {
    padding: 12px;
  }
`;

const StyledTextField = styled(TextField)(() => ({
  "& .MuiInputBase-root": {
    width: "100%",
    margin: 0,
    fontFamily: "Inter",
    fontSize: "12px",
  },
  "& .MuiInputBase-input": {
    padding: "8px",
  },
  "& .MuiInputBase-inputMultiline": {
    padding: "0px",
  },
}));

const FieldWrapper = styled(Box)`
  display: flex;
  flex-direction: column;
  width: 100%;
  gap: 6px;
`;

const StyledLabel = styled("label")`
  font-size: 12px;
  font-family: "Inter";
  font-weight: 500;
  color: ${theme.palette.text.primary};
  display: block;
`;

const StyledHelperText = styled("p")`
  font-size: 12px;
  font-family: "Inter";
  color: ${theme.palette.grey[500]};
  margin: 0;
  line-height: 1.25;
`;

export default EditNamedRange;
