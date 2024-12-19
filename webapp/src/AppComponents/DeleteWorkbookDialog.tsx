import styled from "@emotion/styled";
import { Trash2 } from "lucide-react";
import { forwardRef, useEffect } from "react";
import { theme } from "../theme";

export const DeleteWorkbookDialog = forwardRef<
  HTMLDivElement,
  {
    onClose: () => void;
    onConfirm: () => void;
    workbookName: string;
  }
>((properties, ref) => {
  useEffect(() => {
    const root = document.getElementById("root");
    if (root) {
      root.style.filter = "blur(2px)";
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
      ref={ref}
      tabIndex={-1}
      role="dialog"
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
          >
            Yes, delete workbook
          </DeleteButton>
          <CancelButton onClick={properties.onClose}>Cancel</CancelButton>
        </ButtonGroup>
      </ContentWrapper>
    </DialogWrapper>
  );
});

DeleteWorkbookDialog.displayName = "DeleteWorkbookDialog";

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
  box-shadow: 0px 1px 3px 0px ${theme.palette.common.black}1A;
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
  background-color: ${theme.palette.error.main}1A;
  margin: 12px auto 0 auto;
  color: ${theme.palette.error.main};
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
  color: ${theme.palette.grey["900"]};
`;

const Body = styled.p`
  margin: 0;
  text-align: center;
  color: ${theme.palette.grey["900"]};
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
  color: ${theme.palette.common.white};
  background-color: ${theme.palette.primary.main};
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
    background-color: ${theme.palette.primary.dark};
  }
`;

const DeleteButton = styled(Button)`
  background-color: ${theme.palette.error.main};
  color: ${theme.palette.common.white};
  &:hover {
    background-color: ${theme.palette.error.dark};
  }
`;

const CancelButton = styled(Button)`
  background-color: ${theme.palette.grey["200"]};
  color: ${theme.palette.grey["700"]};
  &:hover {
    background-color: ${theme.palette.grey["300"]};
  }
`;
