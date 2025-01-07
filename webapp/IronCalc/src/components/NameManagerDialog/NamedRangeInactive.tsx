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
          <StyledIconButtonBlack
            onClick={onEdit}
            disabled={!showOptions}
            title={t("name_manager_dialog.edit")}
          >
            <PencilLine size={16} />
          </StyledIconButtonBlack>
          <StyledIconButtonRed
            onClick={onDelete}
            disabled={!showOptions}
            title={t("name_manager_dialog.delete")}
          >
            <Trash2 size={16} />
          </StyledIconButtonRed>
        </IconsWrapper>
      </WrappedLine>
      <Divider />
    </>
  );
}

const StyledIconButtonBlack = styled(IconButton)(({ theme }) => ({
  color: theme.palette.common.black,
  borderRadius: "8px",
  "&:hover": {
    backgroundColor: theme.palette.grey["50"],
  },
}));

const StyledIconButtonRed = styled(IconButton)(({ theme }) => ({
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

const WrappedLine = styled(Box)({
  display: "flex",
  flexDirection: "row",
  alignItems: "center",
  gap: "12px",
  padding: "12px 20px 12px 12px",
});

const StyledDiv = styled("div")(({ theme }) => ({
  fontFamily: theme.typography.fontFamily,
  fontSize: "12px",
  fontWeight: "400",
  color: theme.palette.common.black,
  width: "100%",
  paddingLeft: "8px",
}));

const IconsWrapper = styled(Box)({
  display: "flex",
});

export default NamedRangeInactive;
