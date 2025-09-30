import { Dialog, styled } from "@mui/material";
import { House, Table, TicketsPlane, X } from "lucide-react";
import { useState } from "react";
import TemplatesListItem from "./TemplatesListItem";
import IronCalcIcon from "./ironcalc_icon_white.svg";

function WelcomeDialog(properties: {
  onClose: () => void;
}) {
  const [selectedTemplate, setSelectedTemplate] = useState<string | null>(
    "blank",
  );

  const handleClose = () => {
    properties.onClose();
  };

  const handleTemplateSelect = (templateId: string) => {
    setSelectedTemplate(templateId);
  };

  return (
    <DialogWrapper open={true} onClose={() => {}}>
      <DialogHeader>
        <DialogHeaderTitleWrapper>
          <DialogHeaderLogoWrapper>
            <DialogHeaderLogo src={IronCalcIcon} />
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
      </DialogHeader>
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
        <TemplatesListWrapper>
          <TemplatesListItem
            title="Mortgage calculator"
            description="Estimate payments, interest, and overall cost."
            icon={<House />}
            iconColor="#2F80ED"
            active={selectedTemplate === "mortgage"}
            onClick={() => handleTemplateSelect("mortgage")}
          />
          <TemplatesListItem
            title="Travel expenses tracker"
            description="Track trip costs and stay on budget."
            icon={<TicketsPlane />}
            iconColor="#EB5757"
            active={selectedTemplate === "travel"}
            onClick={() => handleTemplateSelect("travel")}
          />
        </TemplatesListWrapper>
      </DialogContent>
      <DialogFooter>
        <DialogFooterButton>Create workbook</DialogFooterButton>
      </DialogFooter>
    </DialogWrapper>
  );
}

const DialogWrapper = styled(Dialog)`
  font-family: Inter;
  .MuiDialog-paper {
    width: 440px;
    border-radius: 12px;
    margin: 16px;
    border: 1px solid #e0e0e0;;
  }
  .MuiBackdrop-root {
    background-color: rgba(0, 0, 0, 0.4);
  }
`;

const DialogHeader = styled("div")`
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

const DialogHeaderLogoWrapper = styled("div")`
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: center;
  max-width: 20px;
  max-height: 20px;
  background-color: #F2994A;
  padding: 10px;
  margin-bottom: 12px;
  border-radius: 6px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  transform: rotate(-8deg);
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
`;

const DialogHeaderLogo = styled("img")`
  width: 18px;
  height: auto;
`;

const Cross = styled("div")`
  &:hover {
    background-color: #f5f5f5;
  }
  display: flex;
  border-radius: 4px;
  min-height: 24px;
  min-width: 24px;
  cursor: pointer;
  align-items: center;
  justify-content: center;
  svg {
    width: 16px;
    height: 16px;
    stroke-width: 1.5;
  }
`;

const DialogContent = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  max-height: 300px;
  overflow: hidden;
  overflow-y: auto;
`;

const ListTitle = styled("div")`
  font-size: 12px;
  font-weight: 600;
  color: #424242;
`;

const TemplatesListWrapper = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 10px;
`;

const DialogFooter = styled("div")`
  border-top: 1px solid #e0e0e0;
  padding: 16px;
`;

const DialogFooterButton = styled("button")`
  background-color: #F2994A;
  border: none;
  color: #FFF;
  padding: 12px;
  border-radius: 4px;
  cursor: pointer;
  width: 100%;
  font-size: 12px;
  font-family: Inter;
  &:hover {
    background-color: #D68742;
  }
  &:active {
    background-color: #D68742;
  }
`;

export default WelcomeDialog;
