import type { Model } from "@ironcalc/wasm";
import { styled } from "@mui/material";
import { ChevronDown } from "lucide-react";
import { useState } from "react";
import { Fx } from "../../icons";
import { theme } from "../../theme";
import { FORMULA_BAR_HEIGHT } from "../constants";
import Editor from "../Editor/Editor";
import {
  COLUMN_WIDTH_SCALE,
  ROW_HEIGH_SCALE,
} from "../WorksheetCanvas/constants";
import type { WorkbookState } from "../workbookState";
import FormulaBarMenu from "./FormulaBarMenu";

type FormulaBarProps = {
  cellAddress: string;
  formulaValue: string;
  model: Model;
  workbookState: WorkbookState;
  onChange: () => void;
  onTextUpdated: () => void;
  openDrawer: () => void;
  canEdit: boolean;
};

function FormulaBar(properties: FormulaBarProps) {
  const {
    cellAddress,
    formulaValue,
    model,
    onChange,
    onTextUpdated,
    workbookState,
  } = properties;
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  const handleMenuChange = (_option: string): void => {};

  const handleMenuOpenChange = (isOpen: boolean): void => {
    setIsMenuOpen(isOpen);
  };

  return (
    <Container>
      <AddressContainer>
        <FormulaBarMenu
          onChange={handleMenuChange}
          onMenuOpenChange={handleMenuOpenChange}
          openDrawer={properties.openDrawer}
          canEdit={properties.canEdit}
          model={model}
          onUpdate={onChange}
        >
          <CellBarAddress $active={isMenuOpen}>{cellAddress}</CellBarAddress>
          <MenuButton $active={isMenuOpen}>
            <ChevronDown size={16} />
          </MenuButton>
        </FormulaBarMenu>
      </AddressContainer>
      <Divider />
      <FormulaContainer>
        <FormulaSymbolButton>
          <Fx />
        </FormulaSymbolButton>
        <EditorWrapper
          onClick={(event) => {
            const [sheet, row, column] = model.getSelectedCell();
            const editorWidth =
              model.getColumnWidth(sheet, column) * COLUMN_WIDTH_SCALE;
            const editorHeight =
              model.getRowHeight(sheet, row) * ROW_HEIGH_SCALE;
            workbookState.setEditingCell({
              sheet,
              row,
              column,
              text: formulaValue,
              referencedRange: null,
              cursorStart: formulaValue.length,
              cursorEnd: formulaValue.length,
              focus: "formula-bar",
              activeRanges: [],
              mode: "edit",
              editorWidth,
              editorHeight,
            });
            event.stopPropagation();
            event.preventDefault();
          }}
        >
          <Editor
            originalText={formulaValue}
            model={model}
            workbookState={workbookState}
            onEditEnd={() => {
              onChange();
            }}
            onTextUpdated={onTextUpdated}
            type="formula-bar"
          />
        </EditorWrapper>
      </FormulaContainer>
    </Container>
  );
}

const StyledButton = styled("div")`
  display: inline-flex;
  align-items: center;
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
  background-color: ${theme.palette.grey["300"]};
  min-width: 1px;
  height: 16px;
  margin: 0px 16px 0px 8px;
`;

const FormulaContainer = styled("div")`
  margin-left: 0px;
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
  height: ${FORMULA_BAR_HEIGHT}px;
`;

const AddressContainer = styled("div")`
  color: ${theme.palette.common.black};
  font-style: normal;
  font-size: 12px;
  display: flex;
  font-weight: 600;
  flex-grow: row;
  align-items: center;
  gap: 2px;
  border-radius: 4px;
  margin-left: 8px;
  &:hover {
    background-color: ${theme.palette.grey["100"]};
  }
`;

const CellBarAddress = styled("div")<{ $active?: boolean }>`
  width: 100%;
  box-sizing: border-box;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 4px 8px;
  border-radius: 4px 0px 0px 4px;
  background-color: ${(props) =>
    props.$active ? theme.palette.action.selected : "transparent"};
  &:hover {
    background-color: ${theme.palette.grey["300"]};
  }
`;

const MenuButton = styled("div")<{ $active?: boolean }>`
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: 4px 2px;
  border-radius: 0px 4px 4px 0px;
  color: ${theme.palette.common.black};
  background-color: ${(props) =>
    props.$active ? theme.palette.action.selected : "transparent"};
  &:hover {
    background-color: ${theme.palette.grey["300"]};
  }
`;

const EditorWrapper = styled("div")`
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
  font-family: monospace;
`;

export default FormulaBar;
