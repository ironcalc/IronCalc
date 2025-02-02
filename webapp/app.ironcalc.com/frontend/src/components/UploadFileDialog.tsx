import { Dialog, styled } from "@mui/material";
import { BookOpen, FileUp, X } from "lucide-react";
import { type DragEvent, useEffect, useRef, useState } from "react";

function UploadFileDialog(properties: {
  onClose: () => void;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
}) {
  const [hover, setHover] = useState(false);
  const [message, setMessage] = useState("");
  const fileInputRef = useRef<HTMLInputElement>(null);
  const crossRef = useRef<HTMLDivElement>(null);

  const { onModelUpload } = properties;

  useEffect(() => {
    if (crossRef.current) {
      crossRef.current.focus();
    }
    return () => {
      const root = document.getElementById("root");
      if (root) {
        root.style.filter = "none";
      }
    };
  }, []);

  const handleClose = () => {
    properties.onClose();
  };

  const handleDragEnter = (event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.stopPropagation();
    setHover(true);
  };

  const handleDragOver = (event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.stopPropagation();
    event.dataTransfer.dropEffect = "copy";
    setHover(true);
  };

  const handleDragLeave = (event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.stopPropagation();
    setHover(false);
  };

  const handleDrop = (event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.stopPropagation();

    const dt = event.dataTransfer;
    const items = dt.items;

    if (items) {
      // Use DataTransferItemList to access the file(s)
      for (let i = 0; i < items.length; i++) {
        // If dropped items aren't files, skip them
        if (items[i].kind === "file") {
          const file = items[i].getAsFile();
          if (file) {
            handleFileUpload(file);
            return;
          }
        }
      }
    } else {
      const files = dt.files;
      if (files.length > 0) {
        handleFileUpload(files[0]);
      }
    }
  };

  const handleFileUpload = (file: File) => {
    setMessage(`Uploading ${file.name}...`);

    // Read the file as ArrayBuffer
    const reader = new FileReader();
    reader.onload = async () => {
      try {
        await onModelUpload(reader.result as ArrayBuffer, file.name);
        handleClose();
      } catch (e) {
        console.log("error", e);
        setMessage(`${e}`);
      }
    };
    reader.readAsArrayBuffer(file);
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
      <UploadTitle>
        <span style={{ flexGrow: 2, marginLeft: 12 }}>
          Import an .xlsx file
        </span>
        <Cross
          style={{ marginRight: 12 }}
          onClick={handleClose}
          title="Close Dialog"
          ref={crossRef}
          tabIndex={0}
          onKeyDown={(event) => event.key === "Enter" && properties.onClose()}
        >
          <X />
        </Cross>
      </UploadTitle>
      {message === "" ? (
        <DropZone
          onDragEnter={handleDragEnter}
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          onDragExit={handleDragLeave}
          onDrop={handleDrop}
        >
          {!hover ? (
            <>
              <div style={{ flexGrow: 2 }} />
              <div>
                <FileUp
                  style={{
                    width: 16,
                    color: "#EFAA6D",
                    backgroundColor: "#F2994A1A",
                    padding: "2px 6px",
                    borderRadius: 4,
                  }}
                />
              </div>
              <div style={{ fontSize: 12 }}>
                <span style={{ color: "#333333" }}>
                  Drag and drop a file here or{" "}
                </span>
                <input
                  ref={fileInputRef}
                  type="file"
                  multiple
                  accept="*"
                  style={{ display: "none" }}
                  onChange={(event) => {
                    const files = event.target.files;
                    if (files) {
                      for (const file of files) {
                        handleFileUpload(file);
                      }
                    }
                  }}
                />
                <DocLink
                  onClick={() => {
                    if (fileInputRef.current) {
                      fileInputRef.current.click();
                    }
                  }}
                  tabIndex={0}
                  onKeyDown={(event) => {
                    if (event.key === "Enter") {
                      if (fileInputRef.current) {
                        fileInputRef.current.click();
                      }
                    }
                  }}
                >
                  click to browse
                </DocLink>
              </div>
              <div style={{ flexGrow: 2 }} />
            </>
          ) : (
            <>
              <div style={{ flexGrow: 2 }} />
              <div>Drop file here</div>
              <div style={{ flexGrow: 2 }} />
            </>
          )}
        </DropZone>
      ) : (
        <DropZone>
          <>
            <div style={{ flexGrow: 2 }} />
            <div>{message}</div>
            <div style={{ flexGrow: 2 }} />
          </>
        </DropZone>
      )}

      <UploadFooter>
        <BookOpen />
        <UploadFooterLink
          href="https://docs.ironcalc.com/web-application/importing-files.html"
          target="_blank"
          rel="noopener noreferrer"
        >
          Learn more about importing files into IronCalc
        </UploadFooterLink>
      </UploadFooter>
    </DialogWrapper>
  );
}

const DialogWrapper = styled(Dialog)`
  .MuiDialog-paper {
    width: 460px;
  }
  .MuiBackdrop-root {
    background-color: rgba(0, 0, 0, 0.1);
  }
`;

const Cross = styled("div")`
  &:hover {
    background-color: #f5f5f5;
  }
  display: flex;
  border-radius: 4px;
  height: 24px;
  width: 24px;
  cursor: pointer;
  align-items: center;
  justify-content: center;
  svg {
    width: 16px;
    height: 16px;
    stroke-width: 1.5;
  }
`;

const DocLink = styled("span")`
  color: #f2994a;
  text-decoration: none;
  &:hover {
    text-decoration: underline;
  }
`;

const UploadTitle = styled("div")`
  display: flex;
  align-items: center;
  border-bottom: 1px solid #e0e0e0;
  height: 44px;
  font-size: 14px;
  font-weight: 500;
  font-family: Inter;
`;

const UploadFooter = styled("div")`
  height: 44px;
  border-top: 1px solid #e0e0e0;
  color: #757575;
  display: flex;
  align-items: center;
  font-family: Inter;
  gap: 8px;
  padding: 0px 12px;
  svg {
   max-width: 16px;
`;

const UploadFooterLink = styled("a")`
  font-size: 12px;
  font-weight: 400;
  color: #757575;
  text-decoration: none;
  &:hover {
    text-decoration: underline;
  }
`;

const DropZone = styled("div")`
  flex-grow: 2;
  border-radius: 10px;
  height: 160px;
  text-align: center;
  margin: 12px;
  color: #aaa;
  font-family: Inter;
  cursor: pointer;
  background-color: #faebd7;
  border: 1px dashed #efaa6d;
  background: linear-gradient(
    180deg,
    rgba(242, 153, 74, 0.08) 0%,
    rgba(242, 153, 74, 0) 100%
  );
  display: flex;
  flex-direction: column;
  vertical-align: center;
  gap: 16px;
  transition: 0.2s ease-in-out;
  &:hover {
    border: 1px dashed #f2994a;
    transition: 0.2s ease-in-out;
    gap: 8px;
    background: linear-gradient(
      180deg,
      rgba(242, 153, 74, 0.12) 0%,
      rgba(242, 153, 74, 0) 100%
    );
  }
`;

export default UploadFileDialog;
