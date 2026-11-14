import {
  IconButton,
  type IronCalcHandle,
  type Model,
  Tooltip,
} from "@ironcalc/workbook";
import { PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { type RefObject, useLayoutEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { MIN_MAIN_CONTENT_WIDTH_FOR_MOBILE } from "../App";
import { EditMenu } from "./Navigation/EditMenu";
import { ExportPdfDialog } from "./ExportPdfDialog/ExportPdfDialog";
import { FileMenu } from "./Navigation/FileMenu";
import { FormatMenu } from "./Navigation/FormatMenu";
import { HelpMenu } from "./Navigation/HelpMenu";
import { InsertMenu } from "./Navigation/InsertMenu";
import { MobileMenu } from "./Navigation/MobileMenu";
import { ViewMenu } from "./Navigation/ViewMenu";
import { downloadModel } from "./rpc";
import { ShareButton } from "./ShareWorkbook/ShareButton";
import ShareWorkbookDialog from "./ShareWorkbook/ShareDialog";
import { StorageWarning } from "./StorageWarning";
import { type ThemeMode, updateNameSelectedWorkbook } from "./storage";
import { useWindowWidth } from "./useWindowWidth";
import { WorkbookTitle } from "./WorkbookTitle";

import "./file-bar.css";

type OpenMenu = "file" | "edit" | "view" | "insert" | "format" | "help" | null;

export function FileBar(properties: {
  model: Model;
  ironCalcRef: RefObject<IronCalcHandle | null>;
  newModel: () => void;
  newModelFromTemplate: () => void;
  setModel: (key: string) => void;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
  onDelete: () => void;
  isReadOnly: boolean;
  isDrawerOpen: boolean;
  setIsDrawerOpen: (open: boolean) => void;
  onLanguageChange: (language: string) => void;
  themeMode: ThemeMode;
  onThemeModeChange: (mode: ThemeMode) => void;
}) {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [isPdfDialogOpen, setIsPdfDialogOpen] = useState(false);
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

  const handleDownloadPdf = async () => {
    setIsPdfDialogOpen(true);
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
          onDownloadPdf={handleDownloadPdf}
          onModelUpload={properties.onModelUpload}
          onDelete={properties.onDelete}
          isReadOnly={properties.isReadOnly}
          onLanguageChange={properties.onLanguageChange}
          themeMode={properties.themeMode}
          onThemeModeChange={properties.onThemeModeChange}
        />
      ) : (
        <div className="app-ic-file-bar-desktop-menu">
          <FileMenu
            newModel={properties.newModel}
            newModelFromTemplate={properties.newModelFromTemplate}
            onModelUpload={properties.onModelUpload}
            onDownload={handleDownload}
            onDownloadPdf={handleDownloadPdf}
            onDelete={properties.onDelete}
            isReadOnly={properties.isReadOnly}
            isOpen={openMenu === "file"}
            onOpen={() => setOpenMenu("file")}
            onClose={closeMenus}
            onHover={() => openMenu && setOpenMenu("file")}
            onLanguageChange={properties.onLanguageChange}
            themeMode={properties.themeMode}
            onThemeModeChange={properties.onThemeModeChange}
          />
          {!properties.isReadOnly && (
            <>
              <EditMenu
                ironCalcRef={properties.ironCalcRef}
                model={properties.model}
                isOpen={openMenu === "edit"}
                onOpen={() => setOpenMenu("edit")}
                onClose={closeMenus}
                onHover={() => openMenu && setOpenMenu("edit")}
              />
              <ViewMenu
                ironCalcRef={properties.ironCalcRef}
                model={properties.model}
                isOpen={openMenu === "view"}
                onOpen={() => setOpenMenu("view")}
                onClose={closeMenus}
                onHover={() => openMenu && setOpenMenu("view")}
              />
              <InsertMenu
                ironCalcRef={properties.ironCalcRef}
                model={properties.model}
                isOpen={openMenu === "insert"}
                onOpen={() => setOpenMenu("insert")}
                onClose={closeMenus}
                onHover={() => openMenu && setOpenMenu("insert")}
              />
              <FormatMenu
                ironCalcRef={properties.ironCalcRef}
                model={properties.model}
                isOpen={openMenu === "format"}
                onOpen={() => setOpenMenu("format")}
                onClose={closeMenus}
                onHover={() => openMenu && setOpenMenu("format")}
              />
            </>
          )}
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
            updateNameSelectedWorkbook(properties.model, name).catch((e) =>
              console.error("Failed saving new name", e),
            );
          }}
          maxWidth={maxTitleWidth}
          readOnly={properties.isReadOnly}
        />
      </div>
      <div ref={spacerRef} className="app-ic-file-bar-spacer" />
      <div className="app-ic-file-bar-right">
        <StorageWarning />
        <div className="app-ic-file-bar-dialog-container">
          <ShareButton
            onClick={() => setIsDialogOpen(true)}
            disabled={properties.isReadOnly}
          />
          {isDialogOpen && (
            <ShareWorkbookDialog
              onClose={() => setIsDialogOpen(false)}
              model={properties.model}
            />
          )}
          {isPdfDialogOpen && (
            <ExportPdfDialog
              open={isPdfDialogOpen}
              onClose={() => setIsPdfDialogOpen(false)}
              model={properties.model}
            />
          )}
        </div>
      </div>
    </div>
  );
}
