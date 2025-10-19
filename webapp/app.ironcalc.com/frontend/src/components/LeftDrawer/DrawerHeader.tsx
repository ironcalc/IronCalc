import styled from "@emotion/styled";
import { IronCalcIconWhite as IronCalcIcon } from "@ironcalc/workbook";
import { IconButton, Tooltip } from "@mui/material";
import { Plus } from "lucide-react";
import { DialogHeaderLogoWrapper } from "../WelcomeDialog/WelcomeDialog";

interface DrawerHeaderProps {
  onNewModel: () => void;
}

function DrawerHeader({ onNewModel }: DrawerHeaderProps) {
  return (
    <HeaderContainer>
      <LogoWrapper>
        <Logo>
          <IronCalcIcon />
        </Logo>
        <Title>IronCalc</Title>
      </LogoWrapper>
      <Tooltip
        title="New workbook"
        slotProps={{
          popper: {
            modifiers: [
              {
                name: "offset",
                options: {
                  offset: [0, -8],
                },
              },
            ],
          },
        }}
      >
        <AddButton onClick={onNewModel}>
          <PlusIcon />
        </AddButton>
      </Tooltip>
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
  box-sizing: border-box;
  box-shadow: 0 1px 0 0 #e0e0e0;
`;

const LogoWrapper = styled("div")`
  display: flex;
  align-items: center;
  gap: 8px;

`;

const Title = styled("h1")`
  font-size: 14px;
  font-weight: 600;
`;

const Logo = styled(DialogHeaderLogoWrapper)`
  transform: none;
  margin-bottom: 0px;
  padding: 6px;
`;

const AddButton = styled(IconButton)`
  margin-left: 8px;
  height: 32px;
  width: 32px;
  padding: 8px;
  border-radius: 4px;

  svg {
    stroke-width: 2px;
    stroke: #757575;
    width: 16px;
    height: 16px;
  }
  &:hover {
    background-color: #E0E0E0;
  }
  &:active {
    background-color: #BDBDBD;
  }
`;

const PlusIcon = styled(Plus)`
  width: 16px;
  height: 16px;
`;

export default DrawerHeader;
