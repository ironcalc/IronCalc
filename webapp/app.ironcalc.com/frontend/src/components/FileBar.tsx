import "./file-bar.css";
import type { Model } from "@ironcalc/workbook";
import { IconButton, Tooltip } from "@ironcalc/workbook";
import { CloudOff, PanelLeftClose, PanelLeftOpen } from "lucide-react";
import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { MIN_MAIN_CONTENT_WIDTH_FOR_MOBILE } from "../App";
import { MobileMenu } from "./MobileMenu";
import { FileMenu } from "./Navigation/FileMenu";
import { HelpMenu } from "./Navigation/HelpMenu";
import { downloadModel } from "./rpc";
import { ShareButton } from "./ShareWorkbook/ShareButton";
import ShareWorkbookModal from "./ShareWorkbook/ShareWorkbookModal";
import { updateNameSelectedWorkbook } from "./storage";
import { WorkbookTitle } from "./WorkbookTitle";

type OpenMenu = "file" | "help" | null;

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
  onLanguageChange: (language: string) => void;
}) {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [openMenu, setOpenMenu] = useState<OpenMenu>(null);
  const spacerRef = useRef<HTMLDivElement>(null);
  const desktopMenuRef = useRef<HTMLDivElement>(null);
  const [maxTitleWidth, setMaxTitleWidth] = useState(0);
  const width = useWindowWidth();
  const { t } = useTranslation();
  const cloudWarningText1 = t("file_bar.title_input.warning_text1");
  const cloudWarningText2 = t("file_bar.title_input.warning_text2");

  const handleDownload = async () => {
    const model = properties.model;
    const bytes = model.toBytes();
    const fileName = model.getName();
    await downloadModel(bytes, fileName);
  };

  const closeMenus = useCallback(() => setOpenMenu(null), []);

  useEffect(() => {
    if (!openMenu) return;
    const handlePointerDown = (event: PointerEvent) => {
      if (!desktopMenuRef.current?.contains(event.target as Node)) {
        closeMenus();
      }
    };
    document.addEventListener("pointerdown", handlePointerDown, true);
    return () =>
      document.removeEventListener("pointerdown", handlePointerDown, true);
  }, [openMenu, closeMenus]);

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
    <div className="file-bar">
      <Tooltip
        title={t(
          properties.isDrawerOpen
            ? "file_bar.close_sidebar"
            : "file_bar.open_sidebar",
        )}
      >
        <IconButton
          icon={
            properties.isDrawerOpen ? <PanelLeftClose /> : <PanelLeftOpen />
          }
          aria-label={t(
            properties.isDrawerOpen
              ? "file_bar.close_sidebar"
              : "file_bar.open_sidebar",
          )}
          size="md"
          variant="ghost"
          className="file-bar-drawer-button"
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
        <div ref={desktopMenuRef} className="file-bar-desktop-menu">
          <FileMenu
            newModel={properties.newModel}
            newModelFromTemplate={properties.newModelFromTemplate}
            setModel={properties.setModel}
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
      <div className="file-bar-title-wrapper">
        <WorkbookTitle
          name={properties.model.getName()}
          onNameChange={(name) => {
            properties.model.setName(name);
            updateNameSelectedWorkbook(properties.model, name);
            properties.setLocalStorageId((id) => id + 1);
          }}
          maxWidth={maxTitleWidth}
        />
      </div>
      <div className="file-bar-spacer" ref={spacerRef} />
      <div className="file-bar-right">
        <div className="file-bar-cloud-button">
          <CloudOff />
          <div className="file-bar-cloud-tooltip" role="tooltip">
            <span>{cloudWarningText1}</span>
            <strong>{cloudWarningText2}</strong>
          </div>
        </div>
        <div>
          <ShareButton onClick={() => setIsDialogOpen(true)} />
          {isDialogOpen && (
            <ShareWorkbookModal
              onClose={() => setIsDialogOpen(false)}
              onModelUpload={properties.onModelUpload}
              model={properties.model}
            />
          )}
        </div>
      </div>
    </div>
  );
}
