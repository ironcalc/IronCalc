import styled from "@emotion/styled";
import { Alert } from "@mui/material";
import { CircleAlert, X } from "lucide-react";
import { useState } from "react";

const ALERT_DISMISSED_KEY = "localStorageAlertDismissed";

function LocalStorageAlert() {
  const [isAlertVisible, setIsAlertVisible] = useState(
    () => localStorage.getItem(ALERT_DISMISSED_KEY) !== "true",
  );

  const handleClose = () => {
    setIsAlertVisible(false);
    localStorage.setItem(ALERT_DISMISSED_KEY, "true");
  };

  if (!isAlertVisible) {
    return null;
  }

  return (
    <AlertWrapper
      icon={<CircleAlert />}
      action={
        <CloseButton onClick={handleClose}>
          <X />
        </CloseButton>
      }
      sx={{
        padding: 0,
        borderRadius: "8px",
        backgroundColor: "rgba(255, 255, 255, 0.4)",
        backdropFilter: "blur(10px)",
        border: "1px solid #e0e0e0",
        boxShadow: "0px 1px 3px 0px #0000001A",
        fontFamily: "Inter",
        fontWeight: "400",
        lineHeight: "16px",
        zIndex: 1,
      }}
    >
      <AlertTitle>Heads up!</AlertTitle>
      <AlertBody>
        IronCalc stores your data only in your browser's local storage.
      </AlertBody>
      <AlertBody style={{ fontWeight: "600", margin: "6px 0px 0px 0px" }}>
        To keep your work safe, please download your XLSX file regularly.
      </AlertBody>
    </AlertWrapper>
  );
}

const AlertWrapper = styled(Alert)`
  margin: 0;
  .MuiAlert-message {
    font-size: 11px;
    padding: 12px 12px 12px 6px;
    color: #333333;
  }
  .MuiAlert-icon {
    height: 12px;
    width: 12px;
    color: #f2994a;
    margin: 2px 0px 0px 8px;
    padding: 6px 0px;
  }
`;

const AlertTitle = styled("h2")`
  font-size: 11px;
  font-weight: 600;
  line-height: 16px;
  color: #333333;
  margin: 0px 0px 4px 0px;
`;

const AlertBody = styled("p")`
  font-weight: 400;
  line-height: 16px;
  margin: 0;
`;

const CloseButton = styled("button")`
  position: absolute;
  right: 4px;
  top: 4px;
  background: none;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  color: #666666;
  border-radius: 4px;
  transition: background-color 0.2s;
  svg {
    width: 12px;
    height: 12px;
  }
  &:hover {
    background-color: #e0e0e0;
  }
  &:active {
    background-color: #9e9e9e;
  }
`;

export default LocalStorageAlert;
