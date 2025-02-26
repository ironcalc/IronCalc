import styled from "@emotion/styled";
import type { Model } from "@ironcalc/workbook";
import { IronCalcIcon, IronCalcLogo } from "@ironcalc/workbook";
import { useRef, useState } from "react";
import { FileMenu } from "./FileMenu";
import { ShareButton } from "./ShareButton";
import ShareWorkbookDialog from "./ShareWorkbookDialog";
import { WorkbookTitle } from "./WorkbookTitle";
import { downloadModel } from "./rpc";
import { updateNameSelectedWorkbook } from "./storage";

export function FileBar(properties: {
  model: Model;
  newModel: () => void;
  setModel: (key: string) => void;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
  onDelete: () => void;
}) {
  const hiddenInputRef = useRef<HTMLInputElement>(null);
  const [isDialogOpen, setIsDialogOpen] = useState(false);

  return (
    <FileBarWrapper>
      <StyledDesktopLogo />
      <StyledIronCalcIcon />
      <Divider />
      <FileMenu
        newModel={properties.newModel}
        setModel={properties.setModel}
        onModelUpload={properties.onModelUpload}
        onDownload={async () => {
          const model = properties.model;
          const bytes = model.toBytes();
          const fileName = model.getName();
          await downloadModel(bytes, fileName);
        }}
        onDelete={properties.onDelete}
      />
      <HelpButton
        onClick={() => window.open("https://docs.ironcalc.com", "_blank")}
      >
        Help
      </HelpButton>
      <WorkbookTitle
        name={properties.model.getName()}
        onNameChange={(name) => {
          properties.model.setName(name);
          updateNameSelectedWorkbook(properties.model, name);
        }}
      />
      <input
        ref={hiddenInputRef}
        type="text"
        style={{ position: "absolute", left: -9999, top: -9999 }}
      />
      <div style={{ marginLeft: "auto" }} />
      <DialogContainer>
        <ShareButton onClick={() => setIsDialogOpen(true)} />
        {isDialogOpen && (
          <ShareWorkbookDialog
            onClose={() => setIsDialogOpen(false)}
            onModelUpload={properties.onModelUpload}
            model={properties.model}
          />
        )}
      </DialogContainer>
    </FileBarWrapper>
  );
}

const StyledDesktopLogo = styled(IronCalcLogo)`
  width: 120px;
  margin-left: 12px;
  @media (max-width: 769px) {
    display: none;
  }
`;

const StyledIronCalcIcon = styled(IronCalcIcon)`
  width: 36px;
  margin-left: 10px;
  @media (min-width: 769px) {
    display: none;
  }
`;

const HelpButton = styled("div")`
  display: flex;
  align-items: center;
  font-size: 12px;
  font-family: Inter;
  padding: 8px;
  border-radius: 4px;
  cursor: pointer;
  &:hover {
    background-color: #f2f2f2;
  }
`;

const Divider = styled("div")`
  margin: 0px 8px 0px 16px;
  height: 12px;
  border-left: 1px solid #e0e0e0;
`;

const FileBarWrapper = styled("div")`
  height: 60px;
  width: 100%;
  background: #fff;
  display: flex;
  align-items: center;
  border-bottom: 1px solid #e0e0e0;
  position: relative;
  justify-content: space-between;
`;

const DialogContainer = styled("div")`
  position: relative;
  display: inline-block;
  button {
    margin-bottom: 8px;
  }
  .MuiDialog-root {
    position: absolute;
    top: 100%;
    left: 0;
    transform: translateY(8px);
  }
`;
