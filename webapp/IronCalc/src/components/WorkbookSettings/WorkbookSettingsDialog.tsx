import styled from "@emotion/styled";
import { Dialog } from "@mui/material";
import { X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { theme } from "../../theme";

type WorkbookSettingsDialogProps = {
  open: boolean;
  onClose: () => void;
  onExited: () => void;
};

const WorkbookSettingsDialog = (properties: WorkbookSettingsDialogProps) => {
  const { t } = useTranslation();

  const handleClose = () => {
    properties.onClose();
  };

  return (
    <StyledDialog open={properties.open} onClose={properties.onClose}>
      <StyledDialogTitle>
        {t("workbook_settings.title")}
        <Cross
          onClick={handleClose}
          title={t("workbook_settings.close")}
          tabIndex={0}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              properties.onClose();
            }
          }}
        >
          <X />
        </Cross>
      </StyledDialogTitle>
      <StyledDialogContent>
        {/* Settings content will go here */}
      </StyledDialogContent>
    </StyledDialog>
  );
};

const StyledDialog = styled(Dialog)`
  & .MuiPaper-root {
    max-width: 400px;
    min-width: 280px;
    border-radius: 8px;
    padding: 0px;
  }
`;

const StyledDialogTitle = styled("div")`
  display: flex;
  align-items: center;
  height: 44px;
  font-size: 14px;
  font-weight: 500;
  font-family: Inter;
  padding: 0px 12px;
  justify-content: space-between;
  border-bottom: 1px solid ${theme.palette.grey["300"]};
`;

const Cross = styled("div")`
  &:hover {
    background-color: ${theme.palette.grey["50"]};
  }
  display: flex;
  border-radius: 4px;
  height: 24px;
  width: 24px;
  cursor: pointer;
  align-items: center;
  justify-content: center;
  svg {
    width: 16px;
    height: 16px;
    stroke-width: 1.5;
  }
`;

const StyledDialogContent = styled("div")`
  font-size: 12px;
  margin: 12px;
`;

export default WorkbookSettingsDialog;
