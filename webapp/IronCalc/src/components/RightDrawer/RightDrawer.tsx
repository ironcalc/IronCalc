import type { DefinedName, WorksheetProperties } from "@ironcalc/wasm";
import { styled } from "@mui/material/styles";
import type { MouseEvent as ReactMouseEvent, ReactNode } from "react";
import { useCallback, useEffect, useRef, useState } from "react";
import { theme } from "../../theme";
import { TOOLBAR_HEIGHT } from "../constants";
import NamedRanges from "./NamedRanges/NamedRanges";

const DEFAULT_DRAWER_WIDTH = 360;
const MIN_DRAWER_WIDTH = 300;
const MAX_DRAWER_WIDTH = 500;

interface RightDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  width?: number;
  onWidthChange?: (width: number) => void;
  children?: ReactNode;
  showCloseButton?: boolean;
  backgroundColor?: string;
  title?: string;
  definedNameList?: DefinedName[];
  worksheets?: WorksheetProperties[];
  updateDefinedName?: (
    name: string,
    scope: number | undefined,
    newName: string,
    newScope: number | undefined,
    newFormula: string,
  ) => void;
  newDefinedName?: (
    name: string,
    scope: number | undefined,
    formula: string,
  ) => void;
  deleteDefinedName?: (name: string, scope: number | undefined) => void;
  selectedArea?: () => string;
}

const RightDrawer = ({
  isOpen,
  onClose,
  width = DEFAULT_DRAWER_WIDTH,
  onWidthChange,
  children,
  showCloseButton = true,
  title = "Named Ranges",
  definedNameList,
  worksheets,
  updateDefinedName,
  newDefinedName,
  deleteDefinedName,
  selectedArea,
}: RightDrawerProps) => {
  const [drawerWidth, setDrawerWidth] = useState(width);
  const [isResizing, setIsResizing] = useState(false);
  const resizeHandleRef = useRef<HTMLDivElement>(null);

  // Update local width when prop changes
  useEffect(() => {
    setDrawerWidth(width);
  }, [width]);

  const handleMouseDown = useCallback((e: ReactMouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
  }, []);

  useEffect(() => {
    if (!isResizing) return;

    // Prevent text selection during resize
    document.body.style.userSelect = "none";
    document.body.style.cursor = "col-resize";

    const handleMouseMove = (e: MouseEvent) => {
      const newWidth = window.innerWidth - e.clientX;
      const clampedWidth = Math.max(
        MIN_DRAWER_WIDTH,
        Math.min(MAX_DRAWER_WIDTH, newWidth),
      );
      setDrawerWidth(clampedWidth);
      onWidthChange?.(clampedWidth);
    };

    const handleMouseUp = () => {
      setIsResizing(false);
      document.body.style.userSelect = "";
      document.body.style.cursor = "";
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
      document.body.style.userSelect = "";
      document.body.style.cursor = "";
    };
  }, [isResizing, onWidthChange]);

  if (!isOpen) return null;

  return (
    <DrawerContainer $drawerWidth={drawerWidth}>
      <ResizeHandle
        ref={resizeHandleRef}
        onMouseDown={handleMouseDown}
        $isResizing={isResizing}
        aria-label="Resize drawer"
      />
      {children}
      <Divider />
      <DrawerContent>
        <NamedRanges
          title={title}
          onClose={showCloseButton ? onClose : undefined}
          definedNameList={definedNameList}
          worksheets={worksheets}
          updateDefinedName={updateDefinedName}
          newDefinedName={newDefinedName}
          deleteDefinedName={deleteDefinedName}
          selectedArea={selectedArea}
        />
      </DrawerContent>
    </DrawerContainer>
  );
};

type DrawerContainerProps = {
  $drawerWidth: number;
};
const DrawerContainer = styled("div")<DrawerContainerProps>(
  ({ $drawerWidth }) => ({
    position: "absolute",
    overflow: "hidden",
    backgroundColor: theme.palette.common.white,
    right: 0,
    top: `${TOOLBAR_HEIGHT}px`,
    bottom: 0,
    borderLeft: `1px solid ${theme.palette.grey[300]}`,
    width: `${$drawerWidth}px`,
    display: "flex",
    flexDirection: "column",
  }),
);

const Divider = styled("div")({
  height: "1px",
  width: "100%",
  backgroundColor: theme.palette.grey[300],
  margin: "0",
});

const DrawerContent = styled("div")({
  flex: 1,
  height: "100%",
});

const ResizeHandle = styled("div")<{ $isResizing: boolean }>(
  ({ $isResizing }) => ({
    position: "absolute",
    left: 0,
    top: 0,
    bottom: 0,
    width: "4px",
    cursor: "col-resize",
    backgroundColor: $isResizing ? theme.palette.primary.main : "transparent",
    zIndex: 10,
    "&:hover": {
      backgroundColor: theme.palette.primary.main,
      opacity: 0.5,
    },
    transition: $isResizing ? "none" : "background-color 0.2s ease",
  }),
);

export default RightDrawer;
export { DEFAULT_DRAWER_WIDTH, MIN_DRAWER_WIDTH, MAX_DRAWER_WIDTH };
