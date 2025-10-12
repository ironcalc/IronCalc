import { Dialog, styled } from "@mui/material";
import { House, TicketsPlane } from "lucide-react";
import TemplatesListItem from "./TemplatesListItem";

function TemplatesList(props: {
  selectedTemplate: string;
  handleTemplateSelect: (templateId: string) => void;
}) {
  const { selectedTemplate, handleTemplateSelect } = props;
  return (
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
  );
}

export const DialogWrapper = styled(Dialog)`
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

export const Cross = styled("div")`
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

export const DialogContent = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  max-height: 300px;
  overflow: hidden;
  overflow-y: auto;
`;

export const TemplatesListWrapper = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 10px;
`;

export const DialogFooter = styled("div")`
  border-top: 1px solid #e0e0e0;
  padding: 16px;
`;

export const DialogFooterButton = styled("button")`
  background-color: #f2994a;
  border: none;
  color: #fff;
  padding: 12px;
  border-radius: 4px;
  cursor: pointer;
  width: 100%;
  font-size: 12px;
  font-family: Inter;
  &:hover {
    background-color: #d68742;
  }
  &:active {
    background-color: #d68742;
  }
`;

// export default TemplatesDialog;
export default TemplatesList;
