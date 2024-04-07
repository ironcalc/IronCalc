import { Button, styled } from "@mui/material";
import { ChevronDown } from "lucide-react";
import { useState } from "react";
import { Fx } from "../icons";
import { FormulaDialog } from "./formulaDialog";

type FormulaBarProps = {
  cellAddress: string;
  formulaValue: string;
  onChange: (value: string) => void;
};

const formulaBarHeight = 30;
const headerColumnWidth = 30;

function FormulaBar(properties: FormulaBarProps) {
  const [formulaDialogOpen, setFormulaDialogOpen] = useState(false);
  const handleCloseFormulaDialog = () => {
    setFormulaDialogOpen(false);
  };

  return (
    <Container>
      <AddressContainer>
        <CellBarAddress>{properties.cellAddress}</CellBarAddress>
        <StyledButton>
          <ChevronDown />
        </StyledButton>
      </AddressContainer>
      <Divider />
      <FormulaContainer>
        <FormulaSymbolButton>
          <Fx />
        </FormulaSymbolButton>
        <Editor
          onClick={() => {
            setFormulaDialogOpen(true);
          }}
        >
          {properties.formulaValue}
        </Editor>
      </FormulaContainer>
      <FormulaDialog
        isOpen={formulaDialogOpen}
        close={handleCloseFormulaDialog}
        defaultFormula={properties.formulaValue}
        onFormulaChanged={(newName) => {
          properties.onChange(newName);
          setFormulaDialogOpen(false);
        }}
      />
    </Container>
  );
}

const StyledButton = styled(Button)`
  width: 15px;
  min-width: 0px;
  padding: 0px;
  color: inherit;
  font-weight: inherit;
  svg {
    width: 15px;
    height: 15px;
  }
`;

const FormulaSymbolButton = styled(StyledButton)`
  margin-right: 8px;
`;

const Divider = styled("div")`
  background-color: #e0e0e0;
  width: 1px;
  height: 20px;
  margin-left: 16px;
  margin-right: 16px;
`;

const FormulaContainer = styled("div")`
  margin-left: 10px;
  line-height: 22px;
  font-weight: normal;
  width: 100%;
  height: 22px;
  display: flex;
`;

const Container = styled("div")`
  flex-shrink: 0;
  display: flex;
  flex-direction: row;
  align-items: center;
  background: ${(properties): string =>
    properties.theme.palette.background.default};
  height: ${formulaBarHeight}px;
`;

const AddressContainer = styled("div")`
  padding-left: 16px;
  color: #333;
  font-style: normal;
  font-weight: normal;
  font-size: 11px;
  display: flex;
  font-weight: 600;
  flex-grow: row;
  min-width: ${headerColumnWidth}px;
`;

const CellBarAddress = styled("div")`
  width: 100%;
  text-align: "center";
`;

const Editor = styled("div")`
  position: relative;
  width: 100%;
  padding: 0px;
  border-width: 0px;
  outline: none;
  resize: none;
  white-space: pre-wrap;
  vertical-align: bottom;
  overflow: hidden;
  text-align: left;
  span {
    min-width: 1px;
  }
`;

export default FormulaBar;
