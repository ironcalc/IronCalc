import { IronCalcIconWhite as IronCalcIcon } from "@ironcalc/workbook";
import { styled } from "@mui/material";
import { Table, X } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import TemplatesList, {
  Cross,
  DialogContent,
  DialogFooter,
  DialogFooterButton,
  DialogWrapper,
  TemplatesListWrapper,
} from "./TemplatesList";
import TemplatesListItem from "./TemplatesListItem";

function WelcomeDialog(properties: {
  onClose: () => void;
  onSelectTemplate: (templateId: string) => void;
}) {
  const { t } = useTranslation();
  const [selectedTemplate, setSelectedTemplate] = useState<string>("blank");

  const handleClose = () => {
    properties.onClose();
  };

  const handleTemplateSelect = (templateId: string) => {
    setSelectedTemplate(templateId);
  };

  return (
    <DialogWrapper open={true} onClose={() => {}}>
      <DialogWelcomeHeader>
        <DialogHeaderTitleWrapper>
          <DialogHeaderLogoWrapper>
            <IronCalcIcon />
          </DialogHeaderLogoWrapper>
          <DialogHeaderTitle>{t("welcome_dialog.title")}</DialogHeaderTitle>
          <DialogHeaderTitleSubtitle>
            {t("welcome_dialog.subtitle")}
          </DialogHeaderTitleSubtitle>
        </DialogHeaderTitleWrapper>
        <Cross
          onClick={handleClose}
          title={t("welcome_dialog.close_dialog")}
          tabIndex={0}
          onKeyDown={(event) => event.key === "Enter" && properties.onClose()}
        >
          <X />
        </Cross>
      </DialogWelcomeHeader>
      <DialogContent>
        <ListTitle>{t("welcome_dialog.new")}</ListTitle>
        <TemplatesListWrapper>
          <TemplatesListItem
            title={t("welcome_dialog.blank_workbook")}
            description={t("welcome_dialog.blank_workbook_description")}
            icon={<Table />}
            iconColor="#F2994A"
            active={selectedTemplate === "blank"}
            onClick={() => handleTemplateSelect("blank")}
          />
        </TemplatesListWrapper>
        <ListTitle>{t("welcome_dialog.templates")}</ListTitle>
        <TemplatesList
          selectedTemplate={selectedTemplate}
          handleTemplateSelect={handleTemplateSelect}
        />
      </DialogContent>
      <DialogFooter>
        <DialogFooterButton
          onClick={() => properties.onSelectTemplate(selectedTemplate)}
        >
          {t("welcome_dialog.create_workbook")}
        </DialogFooterButton>
      </DialogFooter>
    </DialogWrapper>
  );
}

const DialogWelcomeHeader = styled("div")`
  display: flex;
  flex-direction: row;
  align-items: flex-start;
  border-bottom: 1px solid #e0e0e0;
  padding: 16px;
  font-family: Inter;
`;

const DialogHeaderTitleWrapper = styled("span")`
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  font-size: 14px;
  font-weight: 500;
  padding: 4px 0px;
  gap: 4px;
  width: 100%;
`;

const DialogHeaderTitle = styled("span")`
  font-weight: 700;
`;

const DialogHeaderTitleSubtitle = styled("span")`
  font-size: 12px;
  color: #757575;
`;

export const DialogHeaderLogoWrapper = styled("div")`
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: center;
  max-width: 20px;
  max-height: 20px;
  background-color: #f2994a;
  padding: 10px;
  margin-bottom: 12px;
  border-radius: 6px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  transform: rotate(-8deg);
  user-select: none;

  svg {
    width: 18px;
    height: 18px;
  }
`;

const ListTitle = styled("div")`
  font-size: 12px;
  font-weight: 600;
  color: #424242;
`;

export default WelcomeDialog;
