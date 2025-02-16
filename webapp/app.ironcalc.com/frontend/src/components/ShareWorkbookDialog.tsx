import { Dialog, styled, TextField, Button } from "@mui/material";
import { GlobeLock, Copy, Check } from "lucide-react";
import { useState, useEffect } from "react";
import { shareModel } from "./rpc";
import { Model } from "@ironcalc/workbook";

function ShareWorkbookDialog(properties: {
  onClose: () => void;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
  model?: Model;
}) {
  const [url, setUrl] = useState<string>("");
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    const generateUrl = async () => {
      if (properties.model) {
        const bytes = properties.model.toBytes();
        const fileName = properties.model.getName();
        const hash = await shareModel(bytes, fileName);
        setUrl(`${location.origin}/?model=${hash}`);
      }
    };
    generateUrl();
  }, [properties.model]);

  useEffect(() => {
    let timeoutId: ReturnType<typeof setTimeout>;
    if (copied) {
      timeoutId = setTimeout(() => {
        setCopied(false);
      }, 2000);
    }
    return () => {
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
    };
  }, [copied]);

  const handleClose = () => {
    properties.onClose();
  };

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(url);
      setCopied(true);
    } catch (err) {
      console.error("Failed to copy text: ", err);
    }
  };

  return (
    <DialogWrapper
      open={true}
      tabIndex={0}
      onClose={handleClose}
      onKeyDown={(event) => {
        if (event.code === "Escape") {
          handleClose();
        }
      }}
    >
      <DialogContent>
        <QRCodeWrapper />
        <URLWrapper>
          <StyledTextField
            hiddenLabel
            disabled
            value={url}
            variant="outlined"
            fullWidth
            margin="normal"
            size="small"
            style={{ fontSize: "14px", paddingTop: "0px" }}
          />
          <StyledButton
            variant="contained"
            color="primary"
            size="small"
            onClick={handleCopy}
            style={{ textTransform: "capitalize", fontSize: "14px" }}
          >
            {copied ? <StyledCheck /> : <StyledCopy />}
            {copied ? "Copied!" : "Copy URL"}
          </StyledButton>
        </URLWrapper>
      </DialogContent>

      <UploadFooter>
        <GlobeLock />
        Anyone with the link will be able to access a copy of this workbook
      </UploadFooter>
    </DialogWrapper>
  );
}

const DialogWrapper = styled(Dialog)`
  .MuiDialog-paper {
    width: 440px;
  }
  .MuiBackdrop-root {
    background-color: transparent;
  }
`;

const DialogContent = styled("div")`
  padding: 20px;
  display: flex;
  flex-direction: row;
  gap: 12px;
  height: 80px;
`;

const URLWrapper = styled("div")`
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  justify-content: space-between;
`;

const StyledTextField = styled(TextField)`
  margin: 0px;
  .MuiInputBase-root {
    max-height: 36px;
  }
`;

const StyledButton = styled(Button)`
  display: flex;
  flex-direction: row;
  gap: 4px;
  background-color: #eeeeee;
  height: 36px;
  color: #616161;
  box-shadow: none;
  &:hover {
    background-color: #e0e0e0;
    box-shadow: none;
  }
  &:active {
    background-color: #d4d4d4;
    box-shadow: none;
  }
`;

const StyledCopy = styled(Copy)`
  width: 16px;
`;

const StyledCheck = styled(Check)`
  width: 16px;
`;

const QRCodeWrapper = styled("div")`
  min-height: 80px;
  min-width: 80px;
  background-color: grey;
  border-radius: 4px;
`;

const UploadFooter = styled("div")`
  height: 44px;
  border-top: 1px solid #e0e0e0;
  font-size: 12px;
  font-weight: 400;
  color: #757575;
  display: flex;
  align-items: center;
  font-family: Inter;
  gap: 8px;
  padding: 0px 12px;
  svg {
    max-width: 16px;
  }
`;

export default ShareWorkbookDialog;
