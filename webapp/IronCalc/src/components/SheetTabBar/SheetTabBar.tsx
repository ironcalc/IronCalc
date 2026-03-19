import type { Model } from "@ironcalc/wasm";
import { styled, Tooltip } from "@mui/material";
import { Menu, Plus } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { IronCalcLogo } from "../../icons";
import { NAVIGATION_HEIGHT } from "../constants";
import { getLocaleDisplayName } from "../RightDrawer/RegionalSettings/RegionalSettings";
import { StyledButton } from "../Toolbar/Toolbar";
import type { WorkbookState } from "../workbookState";
import SheetListMenu from "./SheetListMenu";
import SheetTab from "./SheetTab";
import type { SheetOptions } from "./types";

export interface SheetTabBarProps {
  sheets: SheetOptions[];
  selectedIndex: number;
  workbookState: WorkbookState;
  onSheetSelected: (index: number) => void;
  onAddBlankSheet: () => void;
  onSheetColorChanged: (hex: string) => void;
  onSheetRenamed: (name: string) => void;
  onSheetDeleted: () => void;
  onHideSheet: () => void;
  model: Model;
  onOpenRegionalSettings: () => void;
}

function SheetTabBar(props: SheetTabBarProps) {
  const { t } = useTranslation();
  const { workbookState, onSheetSelected, sheets, selectedIndex } = props;
  const [anchorEl, setAnchorEl] = useState<null | HTMLButtonElement>(null);
  const open = Boolean(anchorEl);
  const handleClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    setAnchorEl(event.currentTarget);
  };
  const handleClose = () => {
    setAnchorEl(null);
  };

  const nonHidenSheets = sheets
    .map((s, index) => {
      return {
        state: s.state,
        index,
        name: s.name,
        color: s.color,
        sheetId: s.sheetId,
      };
    })
    .filter((s) => s.state === "visible");

  return (
    <Container>
      <LeftButtonsContainer>
        <Tooltip title={t("navigation.add_sheet")}>
          <StyledButton $pressed={false} onClick={props.onAddBlankSheet}>
            <Plus />
          </StyledButton>
        </Tooltip>
        <Tooltip title={t("navigation.sheet_list")}>
          <StyledButton onClick={handleClick} $pressed={false}>
            <Menu />
          </StyledButton>
        </Tooltip>
      </LeftButtonsContainer>
      <VerticalDivider />
      <Sheets>
        <SheetInner>
          {nonHidenSheets.map((tab) => (
            <SheetTab
              key={tab.sheetId}
              name={tab.name}
              color={tab.color}
              selected={tab.index === selectedIndex}
              onSelected={() => onSheetSelected(tab.index)}
              onColorChanged={(hex: string): void => {
                props.onSheetColorChanged(hex);
              }}
              onRenamed={(name: string): void => {
                props.onSheetRenamed(name);
              }}
              canDelete={nonHidenSheets.length > 1}
              onDeleted={(): void => {
                props.onSheetDeleted();
              }}
              onHideSheet={props.onHideSheet}
              workbookState={workbookState}
            />
          ))}
        </SheetInner>
      </Sheets>
      <RightContainer>
        <Tooltip title={t("regional_settings.open_regional_settings")}>
          <RegionalSettingsButton
            $pressed={false}
            onClick={() => {
              props.onOpenRegionalSettings();
            }}
          >
            {getLocaleDisplayName(props.model.getLocale())}
            <TextDivider />
            {t(
              `regional_settings.language.display_language.${props.model.getLanguage()}`,
            )}
          </RegionalSettingsButton>
        </Tooltip>
        <LogoLink
          onClick={() => window.open("https://www.ironcalc.com", "_blank")}
        >
          <IronCalcLogo />
        </LogoLink>
      </RightContainer>
      <SheetListMenu
        anchorEl={anchorEl}
        open={open}
        onClose={handleClose}
        sheetOptionsList={sheets}
        onSheetSelected={(index) => {
          onSheetSelected(index);
          handleClose();
        }}
        selectedIndex={selectedIndex}
      />
    </Container>
  );
}

// Note I have to specify the font-family in every component that can be considered stand-alone
const Container = styled("div")(({ theme }) => ({
  display: "flex",
  flexDirection: "row",
  position: "absolute",
  bottom: 0,
  left: 0,
  right: 0,
  height: NAVIGATION_HEIGHT,
  alignItems: "center",
  padding: 0,
  fontFamily: "Inter",
  overflow: "hidden",
  backgroundColor: theme.palette.common.white,
  borderTop: `1px solid ${theme.palette.grey[300]}`,
}));

const Sheets = styled("div")({
  flexGrow: 2,
  overflow: "hidden",
  overflowX: "auto",
  scrollbarWidth: "none",
  paddingLeft: 12,
  display: "flex",
  flexDirection: "row",
  height: "100%",
});

const SheetInner = styled("div")({
  display: "flex",
});

const LeftButtonsContainer = styled("div")({
  display: "flex",
  flexDirection: "row",
  alignItems: "center",
  height: "100%",
  gap: 4,
  padding: "0px 12px",
  "@media (max-width: 769px)": {
    padding: "0px 8px",
  },
});

const VerticalDivider = styled("div")(({ theme }) => ({
  height: "100%",
  width: 0,
  "@media (max-width: 769px)": {
    borderRight: `1px solid ${theme.palette.grey[200]}`,
  },
}));

const RightContainer = styled("div")(({ theme }) => ({
  display: "flex",
  flexDirection: "row",
  alignItems: "center",
  color: theme.palette.primary.main,
  height: "100%",
  padding: "0px 8px",
  gap: 4,
  flexShrink: 0,
  width: "auto",
  "@media (max-width: 769px)": {
    display: "none",
  },
}));

const RegionalSettingsButton = styled(StyledButton)(({ theme }) => ({
  minWidth: "fit-content",
  padding: "4px 8px",
  color: theme.palette.grey[600],
  textWrap: "nowrap",
  gap: 8,
}));

const TextDivider = styled("div")(({ theme }) => ({
  width: 1,
  height: "60%",
  backgroundColor: theme.palette.grey[300],
}));

const LogoLink = styled("div")(({ theme }) => ({
  display: "flex",
  alignItems: "center",
  padding: "0px 4px",
  borderRadius: 4,
  maxHeight: 24,
  minHeight: 24,
  cursor: "pointer",
  "& svg": {
    height: 14,
    width: "auto",
  },
  "&:hover": {
    backgroundColor: theme.palette.grey[100],
    transition: "all 0.2s",
    outline: `1px solid ${theme.palette.grey[200]}`,
  },
}));

export default SheetTabBar;
