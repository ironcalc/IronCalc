import type { DefinedName, Model } from "@ironcalc/wasm";
import {
  Box,
  FormControl,
  FormHelperText,
  MenuItem,
  Paper,
  Select,
  styled,
  TextField,
} from "@mui/material";
import { Check, MousePointerClick, Tag } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import { getFullRangeToString } from "../../util";

export interface SaveError {
  nameError: string;
  formulaError: string;
}

interface EditNamedRangeProps {
  name: string;
  scope: string;
  formula: string;
  model: Model;
  onSave: (name: string, scope: string, formula: string) => SaveError;
  onCancel: () => void;
  editingDefinedName: DefinedName | null;
}

// HACK: We are using the text structure of the server error
// to add an error here. This is wrong for several reasons:
// 1. There is no i18n
// 2. Server error messages could change with no warning
export function formatOnSaveError(error: string): SaveError {
  if (error.startsWith("Name: ")) {
    return { formulaError: "", nameError: error.slice(6) };
  } else if (error.startsWith("Formula: ")) {
    return { formulaError: error.slice(9), nameError: "" };
  } else if (error.startsWith("Scope: ")) {
    return { formulaError: "", nameError: error.slice(7) };
  }
  // Fallback for other errors
  return { formulaError: error, nameError: "" };
}

const EditNamedRange = ({
  name: initialName,
  scope: initialScope,
  formula: initialFormula,
  onSave,
  onCancel,
  editingDefinedName,
  model,
}: EditNamedRangeProps) => {
  const { t } = useTranslation();
  const getDefaultName = () => {
    if (initialName) return initialName;
    const rangePrefix = t("name_manager_dialog.default_range_prefix");
    let counter = 1;
    let defaultName = `${rangePrefix}${counter}`;
    const worksheets = model.getWorksheetsProperties();
    const scopeIndex = worksheets.findIndex((s) => s.name === initialScope);
    const newScope = scopeIndex >= 0 ? scopeIndex : undefined;
    const definedNameList = model.getDefinedNameList();

    while (
      definedNameList.some(
        (dn) => dn.name === defaultName && dn.scope === newScope,
      )
    ) {
      counter++;
      defaultName = `${rangePrefix}${counter}`;
    }
    return defaultName;
  };

  const [name, setName] = useState(getDefaultName());
  const [scope, setScope] = useState(initialScope);
  const [formula, setFormula] = useState(initialFormula);
  const [nameError, setNameError] = useState<string>("");
  const [formulaError, setFormulaError] = useState<string>("");

  const isSelected = (value: string) => scope === value;

  // Validate name (format and duplicates)
  useEffect(() => {
    const worksheets = model.getWorksheetsProperties();
    const scopeIndex = worksheets.findIndex((s) => s.name === scope);
    const newScope = scopeIndex >= 0 ? scopeIndex : null;
    try {
      model.isValidDefinedName(name, newScope, formula);
    } catch (e) {
      const message = (e as Error).message;
      if (editingDefinedName && message.includes("already exists")) {
        // Allow the same name if it's the one being edited
        setNameError("");
        setFormulaError("");
        return;
      }
      const { nameError, formulaError } = formatOnSaveError(message);
      setNameError(nameError);
      setFormulaError(formulaError);
      return;
    }
    setNameError("");
    setFormulaError("");
  }, [name, scope, formula, model, editingDefinedName]);

  const hasAnyError = nameError !== "" || formulaError !== "";

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
                      <MenuSpanGrey>{` ${t(
                        "name_manager_dialog.global",
                      )}`}</MenuSpanGrey>
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
                  <MenuSpanGrey>{` ${t(
                    "name_manager_dialog.global",
                  )}`}</MenuSpanGrey>
                </StyledMenuItem>
                {model.getWorksheetsProperties().map((option) => (
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
            <LineWrapper>
              <StyledLabel htmlFor="formula">
                {t("name_manager_dialog.refers_to")}
              </StyledLabel>
              <MousePointerClick
                size={16}
                onClick={() => {
                  const worksheetNames = model
                    .getWorksheetsProperties()
                    .map((s) => s.name);
                  const selectedView = model.getSelectedView();
                  const formula = getFullRangeToString(
                    selectedView,
                    worksheetNames,
                  );
                  setFormula(formula);
                }}
              />
            </LineWrapper>
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
                  setFormulaError("");
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
      <StyledFooter>
        <Button variant="secondary" onClick={onCancel}>
          {t("name_manager_dialog.cancel")}
        </Button>
        <Button
          startIcon={<Check />}
          disabled={hasAnyError}
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
        </Button>
      </StyledFooter>
    </Container>
  );
};

const LineWrapper = styled("div")({
  display: "flex",
  alignItems: "center",
  gap: "8px",
});

const Container = styled("div")({
  height: "100%",
  display: "flex",
  flexDirection: "column",
});

const ContentArea = styled("div")({
  flex: 1,
  overflow: "auto",
});

const MenuSpan = styled("span")<{ $selected?: boolean }>(({ $selected }) => ({
  fontSize: 12,
  fontFamily: "Inter",
  fontWeight: $selected ? "bold" : "normal",
}));

const MenuSpanGrey = styled("span")(({ theme }) => ({
  whiteSpace: "pre",
  fontSize: 12,
  fontFamily: "Inter",
  color: theme.palette.grey[400],
}));

const CheckIcon = () => (
  <Check style={{ width: "16px", height: "16px", marginRight: "8px" }} />
);

const IconPlaceholder = styled("div")({
  width: 16,
  height: 16,
  marginRight: 8,
});

const HeaderBox = styled(Box)(({ theme }) => ({
  fontSize: 14,
  fontFamily: "Inter",
  fontWeight: 600,
  width: "auto",
  gap: 8,
  padding: "24px 12px",
  color: theme.palette.text.primary,
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  textAlign: "center",
  borderBottom: `1px solid ${theme.palette.grey[200]}`,
}));

const HeaderBoxText = styled("span")({
  maxWidth: "100%",
  textOverflow: "ellipsis",
  overflow: "hidden",
  whiteSpace: "nowrap",
});

const HeaderIcon = styled(Box)(({ theme }) => ({
  width: 28,
  height: 28,
  borderRadius: 4,
  backgroundColor: theme.palette.grey[100],
  display: "flex",
  alignItems: "center",
  justifyContent: "center",

  "& svg": {
    width: 16,
    height: 16,
    color: theme.palette.grey[600],
  },
}));

const StyledBox = styled(Box)({
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  gap: 16,
  width: "auto",
  padding: "16px 12px",

  "@media (max-width: 600px)": {
    padding: 12,
  },
});

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

const StyledMenuItem = styled(MenuItem)(({ theme }) => ({
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

const FieldWrapper = styled(Box)({
  display: "flex",
  flexDirection: "column",
  width: "100%",
  gap: 6,
});

const StyledLabel = styled("label")(({ theme }) => ({
  fontSize: "12px",
  fontFamily: "Inter",
  fontWeight: 500,
  color: theme.palette.text.primary,
  display: "block",
}));

const StyledHelperText = styled(FormHelperText)(({ theme }) => ({
  fontSize: "12px",
  fontFamily: "Inter",
  color: theme.palette.grey[500],
  margin: 0,
  marginTop: "6px",
  padding: 0,
  lineHeight: 1.4,
}));

const StyledErrorText = styled(StyledHelperText)(({ theme }) => ({
  color: theme.palette.error.main,
}));

const StyledFooter = styled("div")(({ theme }) => ({
  padding: 8,
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  borderTop: `1px solid ${theme.palette.grey[300]}`,
  gap: 8,
}));

export default EditNamedRange;
