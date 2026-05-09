import type React from "react";
import LocalStorageAlert from "./LocalStorageAlert";
import WorkbookList from "./WorkbookList";

interface DrawerContentProps {
  setModel: (key: string) => void;
  onDelete: (uuid: string) => void;
  searchQuery: string;
  checkedUuids: Set<string>;
  setCheckedUuids: React.Dispatch<React.SetStateAction<Set<string>>>;
}

function DrawerContent({
  setModel,
  onDelete,
  searchQuery,
  checkedUuids,
  setCheckedUuids,
}: DrawerContentProps) {
  return (
    <>
      <div className="app-ic-drawer-content">
        <WorkbookList
          setModel={setModel}
          onDelete={onDelete}
          searchQuery={searchQuery}
          checkedUuids={checkedUuids}
          setCheckedUuids={setCheckedUuids}
        />
      </div>
      <div className="app-ic-drawer-alert-wrapper">
        <LocalStorageAlert />
      </div>
    </>
  );
}

export default DrawerContent;
