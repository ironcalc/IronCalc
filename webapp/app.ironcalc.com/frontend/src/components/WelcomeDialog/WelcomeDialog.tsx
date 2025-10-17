import { IronCalcIconWhite as IronCalcIcon } from "@ironcalc/workbook";
import { styled } from "@mui/material";
import { Table, X } from "lucide-react";
import { useState } from "react";
import TemplatesListItem from "./TemplatesListItem";

import TemplatesList, {
  Cross,
  DialogContent,
  DialogFooter,
  DialogFooterButton,
  DialogWrapper,
  TemplatesListWrapper,
} from "./TemplatesList";

function WelcomeDialog(properties: {
  onClose: () => void;
  onSelectTemplate: (templateId: string) => void;
}) {
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
          <DialogHeaderTitle>Welcome to IronCalc</DialogHeaderTitle>
          <DialogHeaderTitleSubtitle>
            Start with a blank workbook or a ready-made template.
          </DialogHeaderTitleSubtitle>
        </DialogHeaderTitleWrapper>
        <Cross
          onClick={handleClose}
          title="Close Dialog"
          tabIndex={0}
          onKeyDown={(event) => event.key === "Enter" && properties.onClose()}
        >
          <X />
        </Cross>
      </DialogWelcomeHeader>
      <DialogContent>
        <ListTitle>New</ListTitle>
        <TemplatesListWrapper>
          <TemplatesListItem
            title="Blank workbook"
            description="Create from scratch or upload your own file."
            icon={<Table />}
            iconColor="#F2994A"
            active={selectedTemplate === "blank"}
            onClick={() => handleTemplateSelect("blank")}
          />
        </TemplatesListWrapper>
        <ListTitle>Templates</ListTitle>
        <TemplatesList
          selectedTemplate={selectedTemplate}
          handleTemplateSelect={handleTemplateSelect}
        />
      </DialogContent>
      <DialogFooter>
        <DialogFooterButton
          onClick={() => properties.onSelectTemplate(selectedTemplate)}
        >
          Create workbook
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
