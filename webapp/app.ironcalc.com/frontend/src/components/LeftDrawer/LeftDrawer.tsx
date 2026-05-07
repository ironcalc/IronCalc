import { useState } from "react";
import DrawerContent from "./DrawerContent";
import DrawerFooter from "./DrawerFooter";
import DrawerHeader from "./DrawerHeader";
import "./left-drawer.css";

interface LeftDrawerProps {
  open: boolean;
  newModel: () => void;
  setModel: (key: string) => void;
  onDelete: (uuid: string) => void;
  localStorageId: number;
}

function LeftDrawer({ open, newModel, setModel, onDelete }: LeftDrawerProps) {
  const [searchQuery, setSearchQuery] = useState("");

  return (
    <div className={`app-ic-drawer${open ? " app-ic-drawer--open" : ""}`}>
      <div className="app-ic-drawer-paper">
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
    </div>
  );
}

export default LeftDrawer;
