import styled from "@emotion/styled";
import LocalStorageAlert from "./LocalStorageAlert";
import WorkbookList from "./WorkbookList";

interface DrawerContentProps {
  setModel: (key: string) => void;
  onDelete: (uuid: string) => void;
}

function DrawerContent(props: DrawerContentProps) {
  const { setModel, onDelete } = props;

  return (
    <>
      <ContentContainer>
        <WorkbookList setModel={setModel} onDelete={onDelete} />
      </ContentContainer>
      <LocalStorageAlertWrapper>
        <LocalStorageAlert />
      </LocalStorageAlertWrapper>
    </>
  );
}

const ContentContainer = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 16px 12px;
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  font-size: 12px;
`;

const LocalStorageAlertWrapper = styled("div")`
  position: absolute;
  bottom: 56px;
  left: 0;
  right: 0;
  padding: 12px;
`;

export default DrawerContent;
