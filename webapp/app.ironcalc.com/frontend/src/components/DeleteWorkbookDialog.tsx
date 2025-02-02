import styled from "@emotion/styled";
import { Trash2 } from "lucide-react";
import { useEffect, useRef } from "react";

interface DeleteWorkbookDialogProperties {
  onClose: () => void;
  onConfirm: () => void;
  workbookName: string;
}

function DeleteWorkbookDialog(properties: DeleteWorkbookDialogProperties) {
  const deleteButtonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    const root = document.getElementById("root");
    if (root) {
      root.style.filter = "blur(2px)";
    }
    if (deleteButtonRef.current) {
      deleteButtonRef.current.focus();
    }
    return () => {
      const root = document.getElementById("root");
      if (root) {
        root.style.filter = "none";
      }
    };
  }, []);

  return (
    <DialogWrapper
      tabIndex={-1}
      onKeyDown={(event) => {
        if (event.code === "Escape") {
          properties.onClose();
        }
      }}
      aria-labelledby="delete-dialog-title"
      aria-describedby="delete-dialog-description"
    >
      <IconWrapper>
        <Trash2 />
      </IconWrapper>
      <ContentWrapper>
        <Title>Are you sure?</Title>
        <Body>
          The workbook <strong>'{properties.workbookName}'</strong> will be
          permanently deleted. This action cannot be undone.
        </Body>
        <ButtonGroup>
          <DeleteButton
            onClick={() => {
              properties.onConfirm();
              properties.onClose();
            }}
            ref={deleteButtonRef}
          >
            Yes, delete workbook
          </DeleteButton>
          <CancelButton onClick={properties.onClose}>Cancel</CancelButton>
        </ButtonGroup>
      </ContentWrapper>
    </DialogWrapper>
  );
}

DeleteWorkbookDialog.displayName = "DeleteWorkbookDialog";

// some colors taken from the IronCalc palette
const COMMON_WHITE = "#FFF";
const COMMON_BLACK = "#272525";

const ERROR_MAIN = "#EB5757";
const ERROR_DARK = "#CB4C4C";

const GREY_200 = "#EEEEEE";
const GREY_300 = "#E0E0E0";
const GREY_700 = "#616161";
const GREY_900 = "#333333";

const PRIMARY_MAIN = "#F2994A";
const PRIMARY_DARK = "#D68742";

const DialogWrapper = styled.div`
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: white;
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 12px;
  border-radius: 8px;
  box-shadow: 0px 1px 3px 0px ${COMMON_BLACK}1A;
  width: 280px;
  max-width: calc(100% - 40px);
  z-index: 50;
  font-family: "Inter", sans-serif;
`;

const IconWrapper = styled.div`
  display: flex;
  justify-content: center;
  align-items: center;
  width: 36px;
  height: 36px;
  border-radius: 4px;
  background-color: ${ERROR_MAIN}1A;
  margin: 12px auto 0 auto;
  color: ${ERROR_MAIN};
  svg {
    width: 16px;
    height: 16px;
  }
`;

const ContentWrapper = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  word-break: break-word;
`;

const Title = styled.h2`
  margin: 0;
  font-weight: 600;
  font-size: inherit;
  color: ${GREY_900};
`;

const Body = styled.p`
  margin: 0;
  text-align: center;
  color: ${GREY_900};
  font-size: 12px;
`;

const ButtonGroup = styled.div`
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 8px;
  width: 100%;
`;

const Button = styled.button`
  cursor: pointer;
  color: ${COMMON_WHITE};
  background-color: ${PRIMARY_MAIN};
  padding: 0px 10px;
  height: 36px;
  border-radius: 4px;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  text-overflow: ellipsis;
  transition: background-color 150ms;
  &:hover {
    background-color: ${PRIMARY_DARK};
  }
`;

const DeleteButton = styled(Button)`
  background-color: ${ERROR_MAIN};
  color: ${COMMON_WHITE};
  &:hover {
    background-color: ${ERROR_DARK};
  }
`;

const CancelButton = styled(Button)`
  background-color: ${GREY_200};
  color: ${GREY_700};
  &:hover {
    background-color: ${GREY_300};
  }
`;

export default DeleteWorkbookDialog;
