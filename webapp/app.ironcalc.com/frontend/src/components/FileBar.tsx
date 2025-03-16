import styled from "@emotion/styled";
import type { Model } from "@ironcalc/workbook";
import { Button, IconButton } from "@mui/material";
import { PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";
import { DesktopMenu, MobileMenu } from "./FileMenu";
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
  setModel: (key: string) => void;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
  onDelete: () => void;
  isDrawerOpen: boolean;
  setIsDrawerOpen: (open: boolean) => void;
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

  // Common handler functions for both menu types
  const handleDownload = async () => {
    const model = properties.model;
    const bytes = model.toBytes();
    const fileName = model.getName();
    await downloadModel(bytes, fileName);
  };

  return (
    <FileBarWrapper>
      <DrawerButton
        $isDrawerOpen={properties.isDrawerOpen}
        onClick={() => properties.setIsDrawerOpen(!properties.isDrawerOpen)}
        disableRipple
        title="Toggle sidebar"
      >
        {properties.isDrawerOpen ? <PanelLeftClose /> : <PanelLeftOpen />}
      </DrawerButton>
      <DesktopButtonsWrapper>
        <DesktopMenu
          newModel={properties.newModel}
          setModel={properties.setModel}
          onModelUpload={properties.onModelUpload}
          onDownload={handleDownload}
          onDelete={properties.onDelete}
        />
        <FileBarButton
          disableRipple
          onClick={() => window.open("https://docs.ironcalc.com", "_blank")}
        >
          Help
        </FileBarButton>
      </DesktopButtonsWrapper>
      <MobileButtonsWrapper>
        <MobileMenu
          newModel={properties.newModel}
          setModel={properties.setModel}
          onModelUpload={properties.onModelUpload}
          onDownload={handleDownload}
          onDelete={properties.onDelete}
        />
      </MobileButtonsWrapper>
      <Spacer ref={spacerRef} />
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

const WorkbookTitleWrapper = styled("div")`
  position: relative;
`;

// The "Spacer" component occupies as much space as possible between the menu and the share button
const Spacer = styled("div")`
  flex-grow: 1;
`;

const DrawerButton = styled(IconButton)<{ $isDrawerOpen: boolean }>`
  margin-left: 8px;
  height: 32px;
  width: 32px;
  padding: 8px;
  border-radius: 4px;
  cursor: ${(props) => (props.$isDrawerOpen ? "w-resize" : "e-resize")};
  svg {
    stroke-width: 2px;
    stroke: #757575;
    width: 16px;
    height: 16px;
  }
  &:hover {
    background-color: #f2f2f2;
  }
  &:active {
    background-color: #e0e0e0;
  }
`;

// The container must be relative positioned so we can position the title absolutely
const FileBarWrapper = styled("div")`
  position: relative;
  height: 60px;
  min-height: 60px;
  width: 100%;
  background: #fff;
  display: flex;
  align-items: center;
  border-bottom: 1px solid #e0e0e0;
  justify-content: space-between;
  box-sizing: border-box;
`;

const DesktopButtonsWrapper = styled("div")`
  display: flex;
  gap: 4px;
  margin-left: 8px;
  @media (max-width: 600px) {
    display: none;
  }
`;

const MobileButtonsWrapper = styled("div")`
  display: flex;
  gap: 4px;
  @media (min-width: 601px) {
    display: none;
  }
  @media (max-width: 600px) {
    display: flex;
  }
`;

const FileBarButton = styled(Button)`
  display: flex;
  flex-direction: row;
  align-items: center;
  font-size: 12px;
  height: 32px;
  width: auto;
  padding: 4px 8px;
  font-weight: 400;
  min-width: 0px;
  text-transform: capitalize;
  color: #333333;
  &:hover {
    background-color: #f2f2f2;
  }
  &:active {
    background-color: #e0e0e0;
  }
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
