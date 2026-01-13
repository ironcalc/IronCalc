import styled from "@emotion/styled";
import type { Model } from "@ironcalc/workbook";
import { IconButton, Tooltip } from "@mui/material";
import { CloudOff, PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { FileMenu } from "./FileMenu";
import { HelpMenu } from "./HelpMenu";
import LanguageSelector from "./LanguageSelector";
import { downloadModel } from "./rpc";
import { ShareButton } from "./ShareButton";
import ShareWorkbookDialog from "./ShareWorkbookDialog";
import { updateNameSelectedWorkbook } from "./storage";
import { WorkbookTitle } from "./WorkbookTitle";

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
  isDrawerOpen: boolean;
  setIsDrawerOpen: (open: boolean) => void;
  setLocalStorageId: (updater: (id: number) => number) => void;
}) {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const spacerRef = useRef<HTMLDivElement>(null);
  const [maxTitleWidth, setMaxTitleWidth] = useState(0);
  const width = useWindowWidth();
  const { t } = useTranslation();
  const cloudWarningText1 = `${t("file_bar.title_input.warning_text1")}`;
  const cloudWarningText2 = `${t("file_bar.title_input.warning_text2")}`;

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
      <Tooltip
        title={t("file_bar.toggle_sidebar")}
        slotProps={{
          popper: {
            modifiers: [
              {
                name: "offset",
                options: {
                  offset: [0, -8],
                },
              },
            ],
          },
        }}
      >
        <DrawerButton
          onClick={() => properties.setIsDrawerOpen(!properties.isDrawerOpen)}
          disableRipple
        >
          {properties.isDrawerOpen ? <PanelLeftClose /> : <PanelLeftOpen />}
        </DrawerButton>
      </Tooltip>
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
            properties.setLocalStorageId((id) => id + 1);
          }}
          maxWidth={maxTitleWidth}
        />
        <Tooltip
          title={
            <div
              style={{ display: "flex", flexDirection: "column", gap: "4px" }}
            >
              <div>{cloudWarningText1}</div>
              <div style={{ fontWeight: "bold" }}>{cloudWarningText2}</div>
            </div>
          }
          placement="bottom"
          enterTouchDelay={0}
          enterDelay={500}
          slotProps={{
            popper: {
              modifiers: [
                {
                  name: "offset",
                  options: {
                    offset: [0, -10],
                  },
                },
              ],
            },
            tooltip: {
              sx: {
                maxWidth: "240px",
                fontSize: "11px",
                padding: "8px",
                backgroundColor: "#fff",
                color: "#333333",
                borderRadius: "8px",
                border: "1px solid #e0e0e0",
                boxShadow: "0px 1px 3px 0px #0000001A",
                fontFamily: "Inter",
                fontWeight: "400",
                lineHeight: "16px",
              },
            },
          }}
        >
          <CloudButton>
            <CloudOff />
          </CloudButton>
        </Tooltip>
      </WorkbookTitleWrapper>
      <Spacer ref={spacerRef} />
      <LanguageSelectorWrapper>
        <LanguageSelector />
      </LanguageSelectorWrapper>
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

const LanguageSelectorWrapper = styled("div")`
  margin-right: 16px;
`;

// We want the workbook title to be exactly an the center of the page,
// so we need an absolute position
const WorkbookTitleWrapper = styled("div")`
  position: absolute;
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 6px;
  left: 50%;
  transform: translateX(-50%);
`;

const CloudButton = styled("div")`
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: default;
  background-color: transparent;
  border-radius: 4px;
  padding: 8px;
  &:hover {
    background-color: #f2f2f2;
  }
  &:active {
    background-color: #e0e0e0;
  }
  svg {
    width: 16px;
    height: 16px;
    color: #bdbdbd;
  }
`;

// The "Spacer" component occupies as much space as possible between the menu and the share button
const Spacer = styled("div")`
  flex-grow: 1;
`;

// const DrawerButton = styled(IconButton)<{ $isDrawerOpen: boolean }>`
// cursor: ${(props) => (props.$isDrawerOpen ? "w-resize" : "e-resize")};
const DrawerButton = styled(IconButton)`
  margin-left: 8px;
  height: 32px;
  width: 32px;
  padding: 8px;
  border-radius: 4px;

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
  gap: 2px;
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
