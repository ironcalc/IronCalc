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
  const [checkedUuids, setCheckedUuids] = useState<Set<string>>(new Set());

  const handleDeleteChecked = () => {
    for (const uuid of checkedUuids) {
      onDelete(uuid);
    }
    setCheckedUuids(new Set());
  };

  const handleCancelChecked = () => {
    setCheckedUuids(new Set());
  };

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
          checkedUuids={checkedUuids}
          setCheckedUuids={setCheckedUuids}
        />
        <DrawerFooter
          checkedCount={checkedUuids.size}
          onDeleteChecked={handleDeleteChecked}
          onCancelChecked={handleCancelChecked}
        />
      </div>
    </div>
  );
}

export default LeftDrawer;
