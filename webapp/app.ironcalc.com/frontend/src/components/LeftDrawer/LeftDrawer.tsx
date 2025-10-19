import styled from "@emotion/styled";
import { Drawer } from "@mui/material";
import DrawerContent from "./DrawerContent";
import DrawerFooter from "./DrawerFooter";
import DrawerHeader from "./DrawerHeader";

interface LeftDrawerProps {
  open: boolean;
  onClose: () => void;
  newModel: () => void;
  setModel: (key: string) => void;
  onDelete: (uuid: string) => void;
  localStorageId: number;
}

function LeftDrawer({
  open,
  onClose,
  newModel,
  setModel,
  onDelete,
}: LeftDrawerProps) {
  return (
    <DrawerWrapper
      variant="persistent"
      anchor="left"
      open={open}
      onClose={onClose}
      transitionDuration={0}
    >
      <DrawerHeader onNewModel={newModel} />
      <DrawerContent setModel={setModel} onDelete={onDelete} />

      <DrawerFooter />
    </DrawerWrapper>
  );
}

const DrawerWrapper = styled(Drawer)`
  width: 264px;
  height: 100%;
  flex-shrink: 0;
  font-family: "Inter", sans-serif;

  .MuiDrawer-paper {
    width: 264px;
    background-color: #f5f5f5;
    overflow: hidden;
    border-right: 1px solid #e0e0e0;
  }
`;

export default LeftDrawer;
