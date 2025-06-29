import styled from "@emotion/styled";
import { Menu, MenuItem, type PopoverOrigin } from "@mui/material";
import { Plus } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { theme } from "../../theme";
import AdvancedColorPicker from "./AdvancedColorPicker";

type ColorPickerProps = {
  color: string;
  defaultColor: string;
  title: string;
  onChange: (color: string) => void;
  onClose: () => void;
  anchorEl: React.RefObject<HTMLElement | null>;
  anchorOrigin: PopoverOrigin;
  transformOrigin: PopoverOrigin;
  open: boolean;
};

const ColorPicker = ({
  color,
  defaultColor,
  title,
  onChange,
  onClose,
  anchorEl,
  anchorOrigin,
  transformOrigin,
  open,
}: ColorPickerProps) => {
  const [selectedColor, setSelectedColor] = useState<string>(color);
  const [isPickerOpen, setPickerOpen] = useState(false);
  const recentColors = useRef<string[]>([]);
  const { t } = useTranslation();

  useEffect(() => {
    setSelectedColor(color);
  }, [color]);

  const handleColorSelect = (color: string) => {
    if (!recentColors.current.includes(color)) {
      const maxRecentColors = 14;
      recentColors.current = [color, ...recentColors.current].slice(
        0,
        maxRecentColors,
      );
    }
    setSelectedColor(color || theme.palette.common.black);
    onChange(color);
    setPickerOpen(false);
  };

  const handleClose = () => {
    setPickerOpen(false);
    onClose();
  };

  // Colors definitions
  const mainColors = [
    "#FFFFFF",
    "#272525",
    "#1B717E",
    "#3BB68A",
    "#8CB354",
    "#F8CD3C",
    "#F2994A",
    "#EC5753",
    "#523E93",
    "#3358B7",
  ];

  const lightTones = [
    theme.palette.grey[50],
    theme.palette.grey[100],
    theme.palette.grey[200],
    theme.palette.grey[300],
    theme.palette.grey[400],
  ];

  const darkTones = [
    theme.palette.grey[500],
    theme.palette.grey[600],
    theme.palette.grey[700],
    theme.palette.grey[800],
    theme.palette.grey[900],
  ];

  const tealTones = ["#BBD4D8", "#82B1B8", "#498D98", "#1E5A63", "#224348"];
  const greenTones = ["#C4E9DC", "#93D7BF", "#62C5A1", "#358A6C", "#2F5F4D"];
  const limeTones = ["#DDE8CC", "#C0D5A1", "#A3C276", "#6E8846", "#4F5E38"];
  const yellowTones = ["#FDF0C5", "#FBE394", "#F9D764", "#B99A36", "#7A682E"];
  const orangeTones = ["#FBE0C9", "#F8C79B", "#F5AD6E", "#B5763F", "#785334"];
  const redTones = ["#F9CDCB", "#F5A3A0", "#F07975", "#B14845", "#763937"];
  const purpleTones = ["#CBC5DF", "#A095C4", "#7565A9", "#453672", "#382F51"];
  const blueTones = ["#C2CDE9", "#8FA3D7", "#5D79C5", "#30498B", "#2C395F"];

  const toneArrays = [
    lightTones,
    darkTones,
    tealTones,
    greenTones,
    limeTones,
    yellowTones,
    orangeTones,
    redTones,
    purpleTones,
    blueTones,
  ];

  if (!open) {
    return null;
  }

  if (isPickerOpen) {
    return (
      <AdvancedColorPicker
        color={selectedColor}
        onAccept={handleColorSelect}
        onCancel={() => setPickerOpen(false)}
        anchorEl={anchorEl}
        anchorOrigin={anchorOrigin}
        transformOrigin={transformOrigin}
        open={true}
      />
    );
  }

  return (
    <StyledMenu
      anchorEl={anchorEl.current}
      open={true}
      onClose={handleClose}
      anchorOrigin={anchorOrigin}
      transformOrigin={transformOrigin}
    >
      <MenuItemWrapper onClick={() => handleColorSelect(defaultColor)}>
        <MenuItemSquare style={{ backgroundColor: defaultColor }} />
        <MenuItemText>{title}</MenuItemText>
      </MenuItemWrapper>
      <HorizontalDivider />
      <ColorsWrapper>
        <ColorList>
          {mainColors.map((presetColor) => (
            <ColorSwatch
              key={presetColor}
              $color={presetColor}
              onClick={(): void => {
                setSelectedColor(presetColor);
                handleColorSelect(presetColor);
              }}
            />
          ))}
        </ColorList>
        <ColorGrid>
          {toneArrays.map((tones) => (
            <ColorGridCol key={tones.join("-")}>
              {tones.map((presetColor) => (
                <ColorSwatch
                  key={presetColor}
                  $color={presetColor}
                  onClick={(): void => {
                    setSelectedColor(presetColor);
                    handleColorSelect(presetColor);
                  }}
                />
              ))}
            </ColorGridCol>
          ))}
        </ColorGrid>
      </ColorsWrapper>
      <HorizontalDivider />
      <RecentLabel>{t("color_picker.custom")}</RecentLabel>
      <RecentColorsList>
        {recentColors.current.length > 0 ? (
          <>
            {recentColors.current.map((recentColor) => (
              <ColorSwatch
                key={recentColor}
                $color={recentColor}
                onClick={(): void => {
                  setSelectedColor(recentColor);
                  handleColorSelect(recentColor);
                }}
              />
            ))}
          </>
        ) : (
          <EmptyContainer />
        )}
        <StyledPlusButton
          onClick={() => setPickerOpen(true)}
          title={t("color_picker.add")}
        >
          <Plus />
        </StyledPlusButton>
      </RecentColorsList>
    </StyledMenu>
  );
};

// Styled Components
const StyledMenu = styled(Menu)`
  & .MuiPaper-root {
    border-radius: 8px;
    padding: 4px 0px;
    margin-left: -4px;
    max-width: 220px;
  }
  & .MuiList-root {
    padding: 0;
  }
`;

const MenuItemWrapper = styled(MenuItem)`
  display: flex;
  flex-direction: row;
  justify-content: flex-start;
  font-size: 12px;
  gap: 8px;
  width: calc(100% - 8px);
  min-width: 172px;
  margin: 0px 4px 4px 4px;
  border-radius: 4px;
  padding: 8px;
  height: 32px;
`;

const MenuItemText = styled("div")`
  color: #000;
`;

const MenuItemSquare = styled.div`
  width: 16px;
  height: 16px;
  box-sizing: border-box;
  margin-top: 0px;
  border: 1px solid ${theme.palette.grey["300"]};
  border-radius: 4px;
`;

const ColorsWrapper = styled.div`
  display: flex;
  flex-direction: column;
  margin: 4px;
`;

const ColorList = styled.div`
  display: flex;
  flex-wrap: wrap;
  flex-direction: row;
  margin: 8px 8px 0px 8px;
  justify-content: flex-start;
  gap: 4px;
`;

const ColorGrid = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: flex-start;
  margin: 8px;
  gap: 4px;
`;

const ColorGridCol = styled.div`
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  gap: 4px;
`;

const ColorSwatch = styled.button<{ $color: string }>`
  width: 16px;
  height: 16px;
  padding: 0px;
  ${({ $color }): string => {
    if ($color.toUpperCase() === "#FFFFFF") {
      return `border: 1px solid ${theme.palette.grey["300"]};`;
    }
    return `border: 1px solid ${$color};`;
  }}
  background-color: ${({ $color }): string => {
    return $color === "transparent" ? "none" : $color;
  }};
  box-sizing: border-box;
  margin-top: 0px;
  border-radius: 4px;
  &:hover {
    cursor: pointer;
    outline: 1px solid ${theme.palette.grey["300"]};
    outline-offset: 1px;
  }
`;

const HorizontalDivider = styled.div`
  height: 0px;
  width: 100%;
  border-top: 1px solid ${theme.palette.grey["200"]};
`;

const RecentLabel = styled.div`
  font-family: "Inter";
  font-size: 12px;
  font-family: Inter;
  margin: 8px 12px 0px 12px;
  color: ${theme.palette.text.secondary};
`;

const RecentColorsList = styled.div`
  display: flex;
  flex-wrap: wrap;
  flex-direction: row;
  padding: 8px;
  margin: 0px 4px;
  justify-content: flex-start;
  gap: 4px;
`;

const StyledPlusButton = styled("button")`
  display: flex;
  justify-content: center;
  flex-wrap: wrap;
  align-items: center;
  border: none;
  background: none;
  font-size: 12px;
  height: 16px;
  width: 16px;
  margin: 0;
  padding: 0;
  border-radius: 4px;
  svg {
    width: 16px;
    height: 16px;
  }
  &:hover {
    cursor: pointer;
    outline: 1px solid ${theme.palette.grey["300"]};
    outline-offset: 1px;
  }
`;

const EmptyContainer = styled.div`
  display: none;
`;

export default ColorPicker;
