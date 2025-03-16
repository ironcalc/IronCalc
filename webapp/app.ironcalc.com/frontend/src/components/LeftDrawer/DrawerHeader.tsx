import styled from "@emotion/styled";
import { IronCalcLogo } from "@ironcalc/workbook";
import { IconButton } from "@mui/material";
import { Plus } from "lucide-react";

interface DrawerHeaderProps {
  onNewModel: () => void;
}

function DrawerHeader({ onNewModel }: DrawerHeaderProps) {
  return (
    <HeaderContainer>
      <StyledDesktopLogo />
      <AddButton onClick={onNewModel} title="New workbook">
        <PlusIcon />
      </AddButton>
    </HeaderContainer>
  );
}

const HeaderContainer = styled("div")`
  display: flex;
  align-items: center;
  padding: 12px 8px 12px 16px;
  justify-content: space-between;
  max-height: 60px;
  min-height: 60px;
  border-bottom: 1px solid #e0e0e0;
  box-sizing: border-box;
`;

const StyledDesktopLogo = styled(IronCalcLogo)`
  width: 120px;
  height: 28px;
`;

const AddButton = styled(IconButton)`
  background: none;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px;
  height: 32px;
  width: 32px;
  border-radius: 4px;
  margin-left: 10px;
  color: #333333;
  stroke-width: 2px;
  &:hover {
    background-color: #e0e0e0;
  }
`;

const PlusIcon = styled(Plus)`
  width: 16px;
  height: 16px;
`;

export default DrawerHeader;
