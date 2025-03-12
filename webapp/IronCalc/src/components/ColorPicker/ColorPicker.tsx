import styled from "@emotion/styled";
import { Menu, MenuItem, Popover, type PopoverOrigin } from "@mui/material";
import { Check, Plus } from "lucide-react";
import { type JSX, useEffect, useRef, useState } from "react";
import { HexColorInput, HexColorPicker } from "react-colorful";
import { useTranslation } from "react-i18next";
import { theme } from "../../theme";

// Types
type ColorPickerProps = {
  color?: string;
  onChange: (color: string | null) => void;
  onClose?: () => void;
  anchorEl: React.RefObject<HTMLElement | null>;
  anchorOrigin?: PopoverOrigin;
  transformOrigin?: PopoverOrigin;
  open: boolean;
  renderMenuItem?: (
    color: string,
    handleColorSelect: (color: string | null) => void,
  ) => JSX.Element;
};

const colorPickerWidth = 240;

// Main Component
const ColorPicker = ({
  color = theme.palette.common.black,
  onChange,
  onClose,
  anchorEl,
  anchorOrigin,
  transformOrigin,
  open,
  renderMenuItem,
}: ColorPickerProps) => {
  const [selectedColor, setSelectedColor] = useState<string>(color);
  const [isPickerOpen, setPickerOpen] = useState(false);
  const [isMenuOpen, setMenuOpen] = useState(open && !isPickerOpen);
  const recentColors = useRef<string[]>([]);
  const { t } = useTranslation();

  useEffect(() => {
    setSelectedColor(color);
  }, [color]);

  useEffect(() => {
    setMenuOpen(open && !isPickerOpen);
  }, [open, isPickerOpen]);

  const handleColorSelect = (color: string | null) => {
    if (color && !recentColors.current.includes(color)) {
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
    if (onClose) onClose();
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

  // Default menu item renderer
  const defaultRenderMenuItem = (
    color: string,
    handleColorSelect: (color: string | null) => void,
  ) => (
    <MenuItemWrapper onClick={() => handleColorSelect(color)}>
      <MenuItemSquare />
      <MenuItemText>{t("color_picker.default")}</MenuItemText>
    </MenuItemWrapper>
  );

  // Render color picker or menu
  return (
    <>
      {isPickerOpen ? (
        <StylePopover
          open={isPickerOpen}
          onClose={handleClose}
          anchorEl={anchorEl.current}
          anchorOrigin={
            anchorOrigin || { vertical: "bottom", horizontal: "left" }
          }
          transformOrigin={
            transformOrigin || { vertical: "top", horizontal: "left" }
          }
        >
          <ColorPickerDialog>
            <HexColorPicker
              color={selectedColor}
              onChange={(newColor): void => {
                setSelectedColor(newColor);
              }}
            />
            <HorizontalDivider />
            <ColorPickerInput>
              <HexWrapper>
                <HexLabel>{"Hex"}</HexLabel>
                <HexColorInputBox>
                  <HashLabel>{"#"}</HashLabel>
                  <StyledHexColorInput
                    color={selectedColor}
                    onChange={(newColor): void => {
                      setSelectedColor(newColor);
                    }}
                    tabIndex={0}
                  />
                </HexColorInputBox>
              </HexWrapper>
              <Swatch $color={selectedColor} />
            </ColorPickerInput>
            <HorizontalDivider />
            <Buttons>
              <CancelButton onClick={handleClose}>
                {t("color_picker.cancel")}
              </CancelButton>
              <StyledButton
                onClick={(): void => {
                  handleColorSelect(selectedColor);
                  handleClose();
                }}
              >
                <Check />
                {t("color_picker.apply")}
              </StyledButton>
            </Buttons>
          </ColorPickerDialog>
        </StylePopover>
      ) : (
        <StyledMenu
          anchorEl={anchorEl.current}
          open={isMenuOpen}
          onClose={handleClose}
        >
          {(renderMenuItem || defaultRenderMenuItem)(
            theme.palette.common.black,
            handleColorSelect,
          )}
          <HorizontalDivider />
          <ColorsWrapper>
            <ColorList>
              {mainColors.map((presetColor) => (
                <RecentColorButton
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
              {toneArrays.map((tones, index) => (
                <ColorGridCol key={tones.join("-")}>
                  {tones.map((presetColor) => (
                    <RecentColorButton
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
                  <RecentColorButton
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
      )}
    </>
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

const StylePopover = styled(Popover)`
  & .MuiPaper-root {
    border-radius: 8px;
    padding: 0px;
    margin-left: -4px;
    max-width: 220px;
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
  background-color: ${theme.palette.common.black};
  box-sizing: border-box;
  margin-top: 0px;
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

const RecentColorButton = styled.button<{ $color: string }>`
  width: 16px;
  height: 16px;
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

// Color Picker Dialog Styles
const ColorPickerDialog = styled.div`
  background: ${theme.palette.background.default};
  width: ${colorPickerWidth}px;
  padding: 0px;
  display: flex;
  flex-direction: column;
  max-width: 100%;
  & .react-colorful {
    height: 160px;
    width: 100%;
  }
  & .react-colorful__saturation {
    border-bottom: none;
    border-radius: 8px 8px 0px 0px;
  }
  & .react-colorful__hue {
    height: 8px;
    margin: 8px;
    border-radius: 5px;
  }
  & .react-colorful__saturation-pointer {
    width: 14px;
    height: 14px;
  }
  & .react-colorful__hue-pointer {
    width: 7px;
    border-radius: 8px;
    height: 16px;
    width: 16px;
  }
`;

const Buttons = styled.div`
  display: flex;
  justify-content: flex-end;
  margin: 8px;
  gap: 8px;
`;

const StyledButton = styled("div")`
  cursor: pointer;
  width: 100%;
  color: ${theme.palette.primary.contrastText};
  background: ${theme.palette.primary.main};
  padding: 0px 10px;
  height: 28px;
  border-radius: 4px;
  display: flex;
  gap: 4px;
  align-items: center;
  justify-content: center;
  font-family: "Inter";
  font-size: 12px;
  &:hover {
    background: #d68742;
  }
  svg {
    max-width: 12px;
    max-height: 12px;
  }
`;

const CancelButton = styled("div")`
  cursor: pointer;
  width: 100%;
  color: ${theme.palette.grey[700]};
  background: ${theme.palette.grey[200]};
  padding: 0px 10px;
  height: 28px;
  border-radius: 4px;
  display: flex;
  gap: 4px;
  align-items: center;
  justify-content: center;
  font-family: "Inter";
  font-size: 12px;
  &:hover {
    background: ${theme.palette.grey[300]};
  }
  svg {
    max-width: 12px;
    max-height: 12px;
  }
`;

const HashLabel = styled.div`
  margin: auto 0px auto 10px;
  font-size: 13px;
  color: #333;
  font-family: ${theme.typography.button.fontFamily};
`;

const HexLabel = styled.div`
  margin: auto 0px;
  font-size: 12px;
  display: inline-flex;
  font-family: ${theme.typography.button.fontFamily};
`;

const HexColorInputBox = styled.div`
  display: inline-flex;
  flex-grow: 1;
  width: 100%;
  height: 28px;
  border: 1px solid ${theme.palette.grey["300"]};
  border-radius: 5px;
  &:hover {
    border: 1px solid ${theme.palette.grey["600"]};
  }
  &:focus-within {
    outline: 2px solid ${theme.palette.secondary.main};
    outline-offset: 1px;
  }
`;

const StyledHexColorInput = styled(HexColorInput)`
  width: 100%;
  border: none;
  background: transparent;
  outline: none;
  font-family: ${theme.typography.button.fontFamily};
  font-size: 12px;
  text-transform: uppercase;
  text-align: right;
  padding-right: 10px;
  border-radius: 5px;

  &:focus {
    border-color: #4298ef;
  }
`;

const HexWrapper = styled.div`
  display: flex;
  gap: 8px;
  flex-grow: 1;
  & input {
    min-width: 0px;
    border: 0px;
    background: ${theme.palette.background.default};
    outline: none;
    font-family: ${theme.typography.button.fontFamily};
    font-size: 12px;
    text-transform: uppercase;
    text-align: right;
    padding-right: 10px;
    border-radius: 5px;
  }

  & input:focus {
    border-color: #4298ef;
  }
`;

const Swatch = styled.div<{ $color: string }>`
  display: inline-flex;
  ${({ $color }): string => {
    if ($color.toUpperCase() === "#FFFFFF") {
      return `border: 1px solid ${theme.palette.grey["300"]};`;
    }
    return `border: 1px solid ${$color};`;
  }}
  background-color: ${({ $color }): string => $color};
  min-width: 28px;
  height: 28px;
  border-radius: 5px;
`;

const ColorPickerInput = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
  margin: 8px;
  gap: 8px;
`;

export default ColorPicker;
