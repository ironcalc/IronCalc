import { Dialog, styled } from "@mui/material";
import { X } from "lucide-react";
import { useState } from "react";
import TemplatesList, {
  Cross,
  DialogContent,
  DialogFooter,
  DialogFooterButton,
} from "./TemplatesList";

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
      <DialogTemplateHeader>
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
      </DialogTemplateHeader>
      <DialogContent>
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

const DialogTemplateHeader = styled("div")`
  display: flex;
  align-items: center;
  border-bottom: 1px solid #e0e0e0;
  height: 44px;
  font-size: 14px;
  font-weight: 500;
  font-family: Inter;
`;

export default TemplatesDialog;
