import { Box, Divider, IconButton, styled } from "@mui/material";
import { t } from "i18next";
import { PencilLine, Trash2 } from "lucide-react";

interface NamedRangeInactiveProperties {
  name: string;
  scope: string;
  formula: string;
  onDelete: () => void;
  onEdit: () => void;
  showOptions: boolean;
}

function NamedRangeInactive(properties: NamedRangeInactiveProperties) {
  const { name, scope, formula, onDelete, onEdit, showOptions } = properties;

  const scopeName =
    scope === "[global]"
      ? `${t("name_manager_dialog.workbook")} ${t(
          "name_manager_dialog.global",
        )}`
      : scope;

  return (
    <>
      <WrappedLine>
        <StyledDiv>{name}</StyledDiv>
        <StyledDiv>{scopeName}</StyledDiv>
        <StyledDiv>{formula}</StyledDiv>
        <IconsWrapper>
          <StyledIconButtonBlack onClick={onEdit} disabled={!showOptions}>
            <PencilLine size={12} />
          </StyledIconButtonBlack>
          <StyledIconButtonRed onClick={onDelete} disabled={!showOptions}>
            <Trash2 size={12} />
          </StyledIconButtonRed>
        </IconsWrapper>
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
  gap: "12px",
});

const StyledDiv = styled("div")(({ theme }) => ({
  fontFamily: theme.typography.fontFamily,
  fontSize: "12px",
  fontWeight: "400",
  color: theme.palette.common.black,
  width: "153.67px",
  paddingLeft: "8px",
}));

const IconsWrapper = styled(Box)({
  display: "flex",
});

export default NamedRangeInactive;
