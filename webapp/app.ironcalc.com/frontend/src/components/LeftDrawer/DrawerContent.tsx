import styled from "@emotion/styled";
import WorkbookList from "./WorkbookList";

interface DrawerContentProps {
  setModel: (key: string) => void;
  onDelete: (uuid: string) => void;
}

function DrawerContent(props: DrawerContentProps) {
  const { setModel, onDelete } = props;

  return (
    <ContentContainer>
      <WorkbookList setModel={setModel} onDelete={onDelete} />
    </ContentContainer>
  );
}

const ContentContainer = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 16px 12px;
  height: 100%;
  overflow: scroll;
  font-size: 12px;
`;

export default DrawerContent;
