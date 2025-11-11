import type { DefinedName, WorksheetProperties } from "@ironcalc/wasm";
import {
  Box,
  FormControl,
  FormHelperText,
  MenuItem,
  Paper,
  Select,
  TextField,
  styled,
} from "@mui/material";
import { t } from "i18next";
import { Check, Tag } from "lucide-react";
import { useEffect, useState } from "react";
import { theme } from "../../../theme";
import { Footer, NewButton } from "./NamedRanges";

export interface SaveError {
  nameError?: string;
  formulaError?: string;
}

interface EditNamedRangeProps {
  worksheets: WorksheetProperties[];
  name: string;
  scope: string;
  formula: string;
  onSave: (name: string, scope: string, formula: string) => SaveError;
  onCancel: () => void;
  definedNameList: DefinedName[];
  editingDefinedName: DefinedName | null;
}

function EditNamedRange({
  worksheets,
  name: initialName,
  scope: initialScope,
  formula: initialFormula,
  onSave,
  onCancel,
  definedNameList,
  editingDefinedName,
}: EditNamedRangeProps) {
  const getDefaultName = () => {
    if (initialName) return initialName;
    let counter = 1;
    let defaultName = `Range${counter}`;
    const scopeIndex = worksheets.findIndex((s) => s.name === initialScope);
    const newScope = scopeIndex >= 0 ? scopeIndex : undefined;

    while (
      definedNameList.some(
        (dn) => dn.name === defaultName && dn.scope === newScope,
      )
    ) {
      counter++;
      defaultName = `Range${counter}`;
    }
    return defaultName;
  };

  const [name, setName] = useState(getDefaultName());
  const [scope, setScope] = useState(initialScope);
  const [formula, setFormula] = useState(initialFormula);
  const [nameError, setNameError] = useState<string | undefined>(undefined);
  const [formulaError, setFormulaError] = useState<string | undefined>(
    undefined,
  );

  const isSelected = (value: string) => scope === value;

  // Validate name (format and duplicates)
  useEffect(() => {
    const trimmed = name.trim();
    let error: string | undefined;

    if (!trimmed) {
      error = t("name_manager_dialog.errors.range_name_required");
    } else if (trimmed.includes(" ")) {
      error = t("name_manager_dialog.errors.name_cannot_contain_spaces");
    } else if (/^\d/.test(trimmed)) {
      error = t("name_manager_dialog.errors.name_cannot_start_with_number");
    } else if (!/^[a-zA-Z_][a-zA-Z0-9_.]*$/.test(trimmed)) {
      error = t("name_manager_dialog.errors.name_invalid_characters");
    } else {
      // Check for duplicates only if format is valid
      const scopeIndex = worksheets.findIndex((s) => s.name === scope);
      const newScope = scopeIndex >= 0 ? scopeIndex : undefined;
      const existing = definedNameList.find(
        (dn) =>
          dn.name === trimmed &&
          dn.scope === newScope &&
          !(
            editingDefinedName?.name === dn.name &&
            editingDefinedName?.scope === dn.scope
          ),
      );
      if (existing) {
        error = t("name_manager_dialog.errors.name_already_exists");
      }
    }

    setNameError(error);
  }, [name, scope, definedNameList, editingDefinedName, worksheets]);

  const hasAnyError = nameError !== undefined || formulaError !== undefined;

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
            <FormControl fullWidth size="small" error={!!nameError}>
              <StyledTextField
                autoFocus={true}
                id="name"
                variant="outlined"
                size="small"
                margin="none"
                placeholder={t("name_manager_dialog.enter_range_name")}
                fullWidth
                error={!!nameError}
                value={name}
                onChange={(e) => setName(e.target.value)}
                onKeyDown={(e) => e.stopPropagation()}
                onClick={(e) => e.stopPropagation()}
              />
              {nameError && <StyledErrorText>{nameError}</StyledErrorText>}
            </FormControl>
          </FieldWrapper>
          <FieldWrapper>
            <StyledLabel htmlFor="scope">
              {t("name_manager_dialog.scope_label")}
            </StyledLabel>
            <FormControl fullWidth size="small">
              <StyledSelect
                id="scope"
                value={scope}
                onChange={(event) => {
                  setScope(event.target.value as string);
                }}
                renderValue={(value: unknown) => {
                  const stringValue = value as string;
                  return stringValue === "[Global]" ? (
                    <>
                      <MenuSpan>{t("name_manager_dialog.workbook")}</MenuSpan>
                      <MenuSpanGrey>{` ${t("name_manager_dialog.global")}`}</MenuSpanGrey>
                    </>
                  ) : (
                    stringValue
                  );
                }}
                MenuProps={{
                  PaperProps: {
                    component: StyledMenuPaper,
                  },
                  anchorOrigin: {
                    vertical: "bottom",
                    horizontal: "center",
                  },
                  transformOrigin: {
                    vertical: "top",
                    horizontal: "center",
                  },
                  marginThreshold: 0,
                }}
              >
                <StyledMenuItem value={"[Global]"}>
                  {isSelected("[Global]") ? <CheckIcon /> : <IconPlaceholder />}
                  <MenuSpan $selected={isSelected("[Global]")}>
                    {t("name_manager_dialog.workbook")}
                  </MenuSpan>
                  <MenuSpanGrey>{` ${t("name_manager_dialog.global")}`}</MenuSpanGrey>
                </StyledMenuItem>
                {worksheets.map((option) => (
                  <StyledMenuItem key={option.name} value={option.name}>
                    {isSelected(option.name) ? (
                      <CheckIcon />
                    ) : (
                      <IconPlaceholder />
                    )}
                    <MenuSpan $selected={isSelected(option.name)}>
                      {option.name}
                    </MenuSpan>
                  </StyledMenuItem>
                ))}
              </StyledSelect>
              <StyledHelperText>
                {t("name_manager_dialog.scope_helper")}
              </StyledHelperText>
            </FormControl>
          </FieldWrapper>
          <FieldWrapper>
            <StyledLabel htmlFor="formula">
              {t("name_manager_dialog.refers_to")}
            </StyledLabel>
            <FormControl fullWidth size="small" error={!!formulaError}>
              <StyledTextField
                id="formula"
                variant="outlined"
                size="small"
                margin="none"
                placeholder={t("name_manager_dialog.enter_formula")}
                fullWidth
                multiline
                rows={3}
                error={!!formulaError}
                value={formula}
                onChange={(e) => {
                  setFormula(e.target.value);
                  setFormulaError(undefined);
                }}
                onKeyDown={(e) => e.stopPropagation()}
                onClick={(e) => e.stopPropagation()}
              />
              {formulaError && (
                <StyledErrorText>{formulaError}</StyledErrorText>
              )}
            </FormControl>
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
          disabled={hasAnyError}
          startIcon={<Check size={16} />}
          onClick={() => {
            const error = onSave(name.trim(), scope, formula);
            if (error.nameError) {
              setNameError(error.nameError);
            }
            if (error.formulaError) {
              setFormulaError(error.formulaError);
            }
          }}
        >
          {t("name_manager_dialog.apply")}
        </NewButton>
      </Footer>
    </Container>
  );
}

const Container = styled("div")({
  height: "100%",
  display: "flex",
  flexDirection: "column",
});

const ContentArea = styled("div")({
  flex: 1,
  overflow: "auto",
});

const MenuSpan = styled("span")<{ $selected?: boolean }>`
  font-size: 12px;
  font-family: "Inter";
  font-weight: ${(props) => (props.$selected ? "bold" : "normal")};
`;

const MenuSpanGrey = styled("span")`
  white-space: pre;
  font-size: 12px;
  font-family: "Inter";
  color: ${theme.palette.grey[400]};
`;

const CheckIcon = () => (
  <Check style={{ width: "16px", height: "16px", marginRight: "8px" }} />
);

const IconPlaceholder = styled("div")`
  width: 16px;
  height: 16px;
  margin-right: 8px;
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
    padding: "8px",
  },
  "& .MuiInputBase-input": {
    padding: "0px",
  },
  "& .MuiInputBase-inputMultiline": {
    padding: "0px",
  },
}));

const StyledSelect = styled(Select)(() => ({
  fontFamily: "Inter",
  fontSize: "12px",
  "& .MuiSelect-select": {
    padding: "8px",
  },
}));

const StyledMenuPaper = styled(Paper)(() => ({
  padding: 4,
  marginTop: "4px",
  "&.MuiPaper-root": {
    borderRadius: "8px",
  },
  "& .MuiList-padding": {
    padding: 0,
  },
  "& .MuiList-root": {
    padding: 0,
  },
}));

const StyledMenuItem = styled(MenuItem)(() => ({
  padding: 8,
  borderRadius: 4,
  display: "flex",
  alignItems: "center",
  "&.Mui-selected": {
    backgroundColor: "transparent",
    "&:hover": {
      backgroundColor: theme.palette.grey[50],
    },
  },
  "&:hover": {
    backgroundColor: theme.palette.grey[50],
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

const StyledHelperText = styled(FormHelperText)(() => ({
  fontSize: "12px",
  fontFamily: "Inter",
  color: theme.palette.grey[500],
  margin: 0,
  marginTop: "6px",
  padding: 0,
  lineHeight: 1.4,
}));

const StyledErrorText = styled(StyledHelperText)(() => ({
  color: theme.palette.error.main,
}));

export default EditNamedRange;
