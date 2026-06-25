import LocalStorageAlert from "./LocalStorageAlert";
import WorkbookList from "./WorkbookList";

interface DrawerContentProps {
  setModel: (key: string) => void;
  onDelete: (uuid: string) => void;
  searchQuery: string;
  checkedUuids: Set<string>;
  onCheckboxClick: (
    uuid: string,
    shiftKey: boolean,
    orderedUuids: string[],
  ) => void;
}

function DrawerContent({
  setModel,
  onDelete,
  searchQuery,
  checkedUuids,
  onCheckboxClick,
}: DrawerContentProps) {
  return (
    <>
      <div className="app-ic-drawer-content">
        <WorkbookList
          setModel={setModel}
          onDelete={onDelete}
          searchQuery={searchQuery}
          checkedUuids={checkedUuids}
          onCheckboxClick={onCheckboxClick}
        />
      </div>
      <div
        className={`app-ic-drawer-alert-wrapper${checkedUuids.size > 0 ? " app-ic-drawer-alert-wrapper--selection" : ""}`}
      >
        <LocalStorageAlert />
      </div>
    </>
  );
}

export default DrawerContent;
