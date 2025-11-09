import type { WorksheetProperties } from "@ironcalc/wasm";
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

  const isSelected = (value: string) => scope === value;

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
            <FormControl fullWidth size="small" error={formulaError}>
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
  marginLeft: 0,
  marginRight: 0,
  padding: 0,
  lineHeight: 1.4,
  "&.MuiFormHelperText-root": {
    marginTop: "6px",
    marginLeft: 0,
    marginRight: 0,
  },
}));

export default EditNamedRange;
