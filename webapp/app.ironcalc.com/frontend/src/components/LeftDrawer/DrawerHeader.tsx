import styled from "@emotion/styled";
import { IronCalcIconWhite as IronCalcIcon } from "@ironcalc/workbook";
import { IconButton, TextField, Tooltip } from "@mui/material";
import { Plus, Search, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { DialogHeaderLogoWrapper } from "../WelcomeDialog/WelcomeDialog";

interface DrawerHeaderProps {
  onNewModel: () => void;
  searchQuery: string;
  setSearchQuery: (value: string) => void;
}

function DrawerHeader({
  onNewModel,
  searchQuery,
  setSearchQuery,
}: DrawerHeaderProps) {
  const { t } = useTranslation();
  const [isSearching, setIsSearching] = useState(false);
  const searchInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isSearching && searchInputRef.current) {
      searchInputRef.current.focus();
    }
  }, [isSearching]);

  return (
    <HeaderContainer>
      <LogoWrapper className={isSearching ? "hidden" : ""}>
        <Logo>
          <IronCalcIcon />
        </Logo>
        <Title>IronCalc</Title>
      </LogoWrapper>

      <ActionsWrapper className={isSearching ? "hidden" : ""}>
        <Tooltip title={t("left_drawer.search_workbook")}>
          <AddButton onClick={() => setIsSearching(true)}>
            <Search />
          </AddButton>
        </Tooltip>

        <Tooltip title={t("left_drawer.new_workbook")}>
          <AddButton onClick={onNewModel}>
            <Plus />
          </AddButton>
        </Tooltip>
      </ActionsWrapper>

      <SearchOverlay className={isSearching ? "active" : ""}>
        <SearchIconWrapper>
          <Search />
        </SearchIconWrapper>

        <SearchInput
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Escape") {
              setSearchQuery("");
              setIsSearching(false);
            }
          }}
          size="small"
          placeholder={t("left_drawer.search_placeholder")}
          variant="standard"
          fullWidth
          InputProps={{
            disableUnderline: true,
            inputRef: searchInputRef,
          }}
        />
        <ClearIcon
          onClick={() => {
            setSearchQuery("");
            setIsSearching(false);
          }}
        >
          <X />
        </ClearIcon>
      </SearchOverlay>
    </HeaderContainer>
  );
}

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

const ActionsWrapper = styled("div")`
  display: flex;
  flex-direction: row;
  gap: 2px;
`;

const AddButton = styled(IconButton)`
  height: 32px;
  width: 32px;
  padding: 8px;
  border-radius: 6px;

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

const HeaderContainer = styled("div")`
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 10px 12px 16px;
  height: 60px;
  box-sizing: border-box;
  box-shadow: 0 1px 0 0 #e0e0e0;

  .hidden {
    opacity: 0;
    pointer-events: none;
  }
`;

const SearchOverlay = styled("div")`
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 20px;

  opacity: 0;
  transform: translateY(-4px);
  pointer-events: none;

  &.active {
    opacity: 1;
    transform: translateY(0);
    pointer-events: auto;
  }
`;

const SearchIconWrapper = styled("div")`
  display: flex;
  align-items: center;

  svg {
    stroke: #757575;
    width: 16px;
    height: 16px;
  }
`;

const SearchInput = styled(TextField)`
  flex: 1;
  
  .MuiInputBase-root {
    width: 100%;
    height: 32px;
    font-size: 12px;
  }

  .MuiInputBase-input {
    padding: 0; 
    height: 100%;
    box-sizing: border-box;
  }
`;

const ClearIcon = styled("div")`
  display: flex;
  
  width: 16px;
  cursor: pointer;

  svg {
    stroke: #757575;
    width: 16px;
    height: 16px;
    }

    &:hover svg {
      stroke: #272525;
    }
  }
`;

export default DrawerHeader;
