import LocalStorageAlert from "./LocalStorageAlert";
import WorkbookList from "./WorkbookList";

interface DrawerContentProps {
  setModel: (key: string) => void;
  onDelete: (uuid: string) => void;
  searchQuery: string;
}

function DrawerContent({
  setModel,
  onDelete,
  searchQuery,
}: DrawerContentProps) {
  return (
    <>
      <div className="app-ic-drawer-content">
        <WorkbookList
          setModel={setModel}
          onDelete={onDelete}
          searchQuery={searchQuery}
        />
      </div>
      <div className="app-ic-drawer-alert-wrapper">
        <LocalStorageAlert />
      </div>
    </>
  );
}

export default DrawerContent;
