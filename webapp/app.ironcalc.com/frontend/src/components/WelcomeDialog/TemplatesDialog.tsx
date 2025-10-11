import { Dialog, styled } from "@mui/material";
import { House, TicketsPlane, X } from "lucide-react";
import { useState } from "react";
import TemplatesListItem from "./TemplatesListItem";

function TemplatesDialog(properties: {
  onClose: () => void;
  onSelectTemplate: (templateId: string) => void;
}) {
  const [selectedTemplate, setSelectedTemplate] = useState<string>("");

  const handleClose = () => {
    properties.onClose();
  };

  const handleTemplateSelect = (templateId: string) => {
    setSelectedTemplate(templateId);
  };

  return (
    <DialogWrapper open={true} onClose={() => {}}>
      <DialogHeader>
        <span style={{ flexGrow: 2, marginLeft: 12 }}>Choose a template</span>
        <Cross
          style={{ marginRight: 12 }}
          onClick={handleClose}
          title="Close Dialog"
          tabIndex={0}
          onKeyDown={(event) => event.key === "Enter" && properties.onClose()}
        >
          <X />
        </Cross>
      </DialogHeader>
      <DialogContent>
        <TemplatesListWrapper>
          <TemplatesListItem
            title="Mortgage calculator"
            description="Estimate payments, interest, and overall cost."
            icon={<House />}
            iconColor="#2F80ED"
            active={selectedTemplate === "mortgage_calculator"}
            onClick={() => handleTemplateSelect("mortgage_calculator")}
          />
          <TemplatesListItem
            title="Travel expenses tracker"
            description="Track trip costs and stay on budget."
            icon={<TicketsPlane />}
            iconColor="#EB5757"
            active={selectedTemplate === "travel_expenses_tracker"}
            onClick={() => handleTemplateSelect("travel_expenses_tracker")}
          />
        </TemplatesListWrapper>
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

const DialogWrapper = styled(Dialog)`
    font-family: Inter;
    .MuiDialog-paper {
        width: 440px;
        border-radius: 12px;
        margin: 16px;
        border: 1px solid #e0e0e0;
    }
    .MuiBackdrop-root {
        background-color: rgba(0, 0, 0, 0.4);
    }
`;

const DialogHeader = styled("div")`
    display: flex;
    align-items: center;
    border-bottom: 1px solid #e0e0e0;
    height: 44px;
    font-size: 14px;
    font-weight: 500;
    font-family: Inter;
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

export default TemplatesDialog;
