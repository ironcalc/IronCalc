import {
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  TextField,
} from "@mui/material";
import { useState } from "react";
import { useTranslation } from "react-i18next";

interface FormulaDialogProps {
  isOpen: boolean;
  close: () => void;
  onFormulaChanged: (name: string) => void;
  defaultFormula: string;
}

export const FormulaDialog = (properties: FormulaDialogProps) => {
  const { t } = useTranslation();
  const [formula, setFormula] = useState(properties.defaultFormula);
  return (
    <Dialog open={properties.isOpen} onClose={properties.close}>
      <DialogTitle>{t("formula_input.title")}</DialogTitle>
      <DialogContent dividers>
        <TextField
          defaultValue={formula}
          label={t("formula_input.label")}
          onClick={(event) => event.stopPropagation()}
          onKeyDown={(event) => {
            event.stopPropagation();
          }}
          onChange={(event) => {
            setFormula(event.target.value);
          }}
          spellCheck="false"
        />
      </DialogContent>
      <DialogActions>
        <Button
          onClick={() => {
            properties.onFormulaChanged(formula);
          }}
        >
          {t("formula_input.update")}
        </Button>
      </DialogActions>
    </Dialog>
  );
};
