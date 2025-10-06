import styled from "@emotion/styled";
import type { Model } from "@ironcalc/workbook";
import { IronCalcIcon, IronCalcLogo } from "@ironcalc/workbook";
import { useLayoutEffect, useRef, useState } from "react";
import { FileMenu } from "./FileMenu";
import { HelpMenu } from "./HelpMenu";
import { ShareButton } from "./ShareButton";
import ShareWorkbookDialog from "./ShareWorkbookDialog";
import { WorkbookTitle } from "./WorkbookTitle";
import { downloadModel } from "./rpc";
import { updateNameSelectedWorkbook } from "./storage";

// This hook is used to get the width of the window
function useWindowWidth() {
  const [width, setWidth] = useState(0);
  useLayoutEffect(() => {
    function updateWidth() {
      setWidth(window.innerWidth);
    }
    window.addEventListener("resize", updateWidth);
    updateWidth();
    return () => window.removeEventListener("resize", updateWidth);
  }, []);
  return width;
}

export function FileBar(properties: {
  model: Model;
  newModel: () => void;
  newModelFromTemplate: () => void;
  setModel: (key: string) => void;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
  onDelete: () => void;
}) {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const spacerRef = useRef<HTMLDivElement>(null);
  const [maxTitleWidth, setMaxTitleWidth] = useState(0);
  const width = useWindowWidth();

  // biome-ignore lint/correctness/useExhaustiveDependencies: We need to update the maxTitleWidth when the width changes
  useLayoutEffect(() => {
    const el = spacerRef.current;
    if (el) {
      const bb = el.getBoundingClientRect();
      setMaxTitleWidth(bb.right - bb.left - 50);
    }
  }, [width]);

  return (
    <FileBarWrapper>
      <StyledDesktopLogo />
      <StyledIronCalcIcon />
      <Divider />
      <FileMenu
        newModel={properties.newModel}
        newModelFromTemplate={properties.newModelFromTemplate}
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
      <HelpMenu />
      <WorkbookTitleWrapper>
        <WorkbookTitle
          name={properties.model.getName()}
          onNameChange={(name) => {
            properties.model.setName(name);
            updateNameSelectedWorkbook(properties.model, name);
          }}
          maxWidth={maxTitleWidth}
        />
      </WorkbookTitleWrapper>
      <Spacer ref={spacerRef} />
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

// We want the workbook title to be exactly an the center of the page,
// so we need an absolute position
const WorkbookTitleWrapper = styled("div")`
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
`;

// The "Spacer" component occupies as much space as possible between the menu and the share button
const Spacer = styled("div")`
  flex-grow: 1;
`;

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

const Divider = styled("div")`
  margin: 0px 8px 0px 16px;
  height: 12px;
  border-left: 1px solid #e0e0e0;
`;

// The container must be relative positioned so we can position the title absolutely
const FileBarWrapper = styled("div")`
  position: relative;
  height: 60px;
  width: 100%;
  background: #fff;
  display: flex;
  align-items: center;
  border-bottom: 1px solid #e0e0e0;
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
