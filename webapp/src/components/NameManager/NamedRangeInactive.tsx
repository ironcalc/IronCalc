import type { Model, WorksheetProperties } from "@ironcalc/wasm";
import { Box, Divider, IconButton, styled } from "@mui/material";
import { t } from "i18next";
import { PencilLine, Trash2 } from "lucide-react";

interface NamedRangeInactiveProperties {
  model: Model;
  worksheets: WorksheetProperties[];
  name: string;
  scope?: number;
  formula: string;
  onDelete: () => void;
  onEdit: () => void;
  showOptions: boolean;
}

function NamedRangeInactive(properties: NamedRangeInactiveProperties) {
  const {
    model,
    worksheets,
    name,
    scope,
    formula,
    onDelete,
    onEdit,
    showOptions,
  } = properties;

  const handleDelete = () => {
    try {
      model.deleteDefinedName(name, scope);
    } catch (error) {
      console.log("DefinedName delete failed", error);
    }
    onDelete();
  };

  const scopeName =
    worksheets.find((sheet, index) => index === scope)?.name ||
    t("name_manager_dialog.workbook");

  return (
    <>
      <WrappedLine>
        <StyledDiv>{name}</StyledDiv>
        <StyledDiv>{scopeName}</StyledDiv>
        <StyledDiv>{formula}</StyledDiv>
        <WrappedIcons>
          <StyledIconButtonBlack onClick={onEdit} disabled={!showOptions}>
            <PencilLine size={12} />
          </StyledIconButtonBlack>
          <StyledIconButtonRed onClick={handleDelete} disabled={!showOptions}>
            <Trash2 size={12} />
          </StyledIconButtonRed>
        </WrappedIcons>
      </WrappedLine>
      <Divider />
    </>
  );
}

const StyledIconButtonBlack = styled(IconButton)(({ theme }) => ({
  color: theme.palette.common.black,
}));

const StyledIconButtonRed = styled(IconButton)(({ theme }) => ({
  color: theme.palette.error.main,
  "&.Mui-disabled": {
    opacity: 0.6,
    color: theme.palette.error.light,
  },
}));

const WrappedLine = styled(Box)({
  display: "flex",
  height: "28px",
  alignItems: "center",
});

const StyledDiv = styled("div")(({ theme }) => ({
  fontFamily: theme.typography.fontFamily,
  fontSize: "12px",
  fontWeight: "400",
  color: theme.palette.common.black,
  width: "171px",
}));

const WrappedIcons = styled(Box)({
  display: "flex",
  gap: "0px",
});

export default NamedRangeInactive;
