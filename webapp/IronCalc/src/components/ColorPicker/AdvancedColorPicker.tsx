import styled from "@emotion/styled";
import { Popover, type PopoverOrigin } from "@mui/material";
import { Check } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { HexColorInput, HexColorPicker } from "react-colorful";
import { useTranslation } from "react-i18next";
import { theme } from "../../theme";

type AdvancedColorPickerProps = {
  color: string;
  onAccept: (color: string) => void;
  onCancel: () => void;
  anchorEl: React.RefObject<HTMLElement | null>;
  anchorOrigin: PopoverOrigin;
  transformOrigin: PopoverOrigin;
  open: boolean;
};

const AdvancedColorPicker = ({
  color,
  onAccept,
  onCancel,
  anchorEl,
  anchorOrigin,
  transformOrigin,
  open,
}: AdvancedColorPickerProps) => {
  const [selectedColor, setSelectedColor] = useState<string>(color);
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
    setSelectedColor(color);
    onAccept(color);
  };

  return (
    <StylePopover
      open={open}
      onClose={onCancel}
      anchorEl={anchorEl.current}
      anchorOrigin={anchorOrigin}
      transformOrigin={transformOrigin}
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
          <CancelButton onClick={onCancel}>
            {t("color_picker.cancel")}
          </CancelButton>
          <StyledButton
            onClick={(): void => {
              handleColorSelect(selectedColor);
              onCancel();
            }}
          >
            <Check />
            {t("color_picker.apply")}
          </StyledButton>
        </Buttons>
      </ColorPickerDialog>
    </StylePopover>
  );
};

const StylePopover = styled(Popover)`
  & .MuiPaper-root {
    border-radius: 8px;
    padding: 0px;
    margin-left: -4px;
    max-width: 220px;
  }
`;

const HorizontalDivider = styled.div`
  height: 0px;
  width: 100%;
  border-top: 1px solid ${theme.palette.grey["200"]};
`;

// Color Picker Dialog Styles
const ColorPickerDialog = styled.div`
  background: ${theme.palette.background.default};
  width: 240px;
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

export default AdvancedColorPicker;
