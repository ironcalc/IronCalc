import "./left-drawer.css";
import { useState } from "react";
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

function LeftDrawer({ open, newModel, setModel, onDelete }: LeftDrawerProps) {
  const [searchQuery, setSearchQuery] = useState("");

  return (
    <div className={`left-drawer${open ? "" : " left-drawer--closed"}`}>
      <DrawerHeader
        onNewModel={newModel}
        searchQuery={searchQuery}
        setSearchQuery={setSearchQuery}
      />
      <DrawerContent
        setModel={setModel}
        onDelete={onDelete}
        searchQuery={searchQuery}
      />
      <DrawerFooter />
    </div>
  );
}

export default LeftDrawer;
