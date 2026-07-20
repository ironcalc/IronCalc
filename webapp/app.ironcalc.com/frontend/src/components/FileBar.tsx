import {
  type CollabProvider,
  IconButton,
  type Model,
  Tooltip,
} from "@ironcalc/workbook";
import { PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { MIN_MAIN_CONTENT_WIDTH_FOR_MOBILE } from "../App";
import { CollabControls } from "./Collab/CollabControls";
import { FileMenu } from "./Navigation/FileMenu";
import { HelpMenu } from "./Navigation/HelpMenu";
import { MobileMenu } from "./Navigation/MobileMenu";
import { downloadModel } from "./rpc";
import { ShareButton } from "./ShareWorkbook/ShareButton";
import ShareWorkbookDialog from "./ShareWorkbook/ShareDialog";
import { StorageWarning } from "./StorageWarning";
import { updateNameSelectedWorkbook } from "./storage";
import { useWindowWidth } from "./useWindowWidth";
import { WorkbookTitle } from "./WorkbookTitle";

import "./file-bar.css";

type OpenMenu = "file" | "help" | null;

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
  onLanguageChange: (language: string) => void;
  collabProvider: CollabProvider | null;
  onStartCollaboration: () => void;
}) {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [openMenu, setOpenMenu] = useState<OpenMenu>(null);
  const spacerRef = useRef<HTMLDivElement>(null);
  const [maxTitleWidth, setMaxTitleWidth] = useState(0);
  const width = useWindowWidth();
  const { t } = useTranslation();
  const handleDownload = async () => {
    const model = properties.model;
    const bytes = model.toBytes();
    const fileName = model.getName();
    await downloadModel(bytes, fileName);
  };

  const closeMenus = () => setOpenMenu(null);
  const drawerLabel = t(
    properties.isDrawerOpen
      ? "file_bar.close_sidebar"
      : "file_bar.open_sidebar",
  );

  // biome-ignore lint/correctness/useExhaustiveDependencies: We need to update the maxTitleWidth when the width changes
  useLayoutEffect(() => {
    const el = spacerRef.current;
    if (el) {
      const bb = el.getBoundingClientRect();
      setMaxTitleWidth(bb.right - bb.left - 50);
    }
  }, [width]);

  const isMobile = width < MIN_MAIN_CONTENT_WIDTH_FOR_MOBILE;

  return (
    <div className="app-ic-file-bar">
      <Tooltip title={drawerLabel}>
        <IconButton
          icon={
            properties.isDrawerOpen ? <PanelLeftClose /> : <PanelLeftOpen />
          }
          aria-label={drawerLabel}
          size="md"
          className="app-ic-file-bar-drawer-button"
          onClick={() => properties.setIsDrawerOpen(!properties.isDrawerOpen)}
        />
      </Tooltip>
      {isMobile ? (
        <MobileMenu
          newModel={properties.newModel}
          newModelFromTemplate={properties.newModelFromTemplate}
          onDownload={handleDownload}
          onModelUpload={properties.onModelUpload}
          onDelete={properties.onDelete}
          onLanguageChange={properties.onLanguageChange}
        />
      ) : (
        <div className="app-ic-file-bar-desktop-menu">
          <FileMenu
            newModel={properties.newModel}
            newModelFromTemplate={properties.newModelFromTemplate}
            onModelUpload={properties.onModelUpload}
            onDownload={handleDownload}
            onDelete={properties.onDelete}
            isOpen={openMenu === "file"}
            onOpen={() => setOpenMenu("file")}
            onClose={closeMenus}
            onHover={() => openMenu && setOpenMenu("file")}
            onLanguageChange={properties.onLanguageChange}
          />
          <HelpMenu
            isOpen={openMenu === "help"}
            onOpen={() => setOpenMenu("help")}
            onClose={closeMenus}
            onHover={() => openMenu && setOpenMenu("help")}
          />
        </div>
      )}
      <div className="app-ic-file-bar-title-wrapper">
        <WorkbookTitle
          name={properties.model.getName()}
          onNameChange={(name) => {
            properties.model.setName(name);
            if (!properties.collabProvider) {
              // Collab sessions live on the relay server; the "selected"
              // localStorage entry is some unrelated local workbook.
              updateNameSelectedWorkbook(properties.model, name);
            }
            properties.setLocalStorageId((id) => id + 1);
          }}
          maxWidth={maxTitleWidth}
        />
      </div>
      <div ref={spacerRef} className="app-ic-file-bar-spacer" />
      <div className="app-ic-file-bar-right">
        <StorageWarning />
        <CollabControls
          provider={properties.collabProvider}
          onStartCollaboration={properties.onStartCollaboration}
        />
        <div className="app-ic-file-bar-dialog-container">
          <ShareButton onClick={() => setIsDialogOpen(true)} />
          {isDialogOpen && (
            <ShareWorkbookDialog
              onClose={() => setIsDialogOpen(false)}
              model={properties.model}
            />
          )}
        </div>
      </div>
    </div>
  );
}
