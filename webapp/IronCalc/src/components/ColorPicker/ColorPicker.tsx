import styled from "@emotion/styled";
import Popover, { type PopoverOrigin } from "@mui/material/Popover";
import { Check } from "lucide-react";
import type React from "react";
import { useEffect, useRef, useState } from "react";
import { HexColorInput, HexColorPicker } from "react-colorful";
import { useTranslation } from "react-i18next";
import { theme } from "../../theme";

type ColorPickerProps = {
  color: string;
  onChange: (color: string) => void;
  onClose: () => void;
  anchorEl: React.RefObject<HTMLElement | null>;
  anchorOrigin?: PopoverOrigin;
  transformOrigin?: PopoverOrigin;
  open: boolean;
};

const colorPickerWidth = 240;
const colorfulHeight = 240;

const ColorPicker = (properties: ColorPickerProps) => {
  const [color, setColor] = useState<string>(properties.color);
  const recentColors = useRef<string[]>([]);

  const { t } = useTranslation();

  const closePicker = (newColor: string): void => {
    const maxRecentColors = 14;
    const colors = recentColors.current.filter((c) => c !== newColor);
    recentColors.current = [newColor, ...colors].slice(0, maxRecentColors);
    properties.onChange(newColor);
  };

  const handleClose = (): void => {
    properties.onClose();
  };

  useEffect(() => {
    setColor(properties.color);
  }, [properties.color]);

  const presetColors = [
    "#FFFFFF",
    "#1B717E",
    "#59B9BC",
    "#3BB68A",
    "#8CB354",
    "#F8CD3C",
    "#F2994A",
    "#EC5753",
    "#D03627",
    "#523E93",
    "#3358B7",
  ];

  return (
    <Popover
      open={properties.open}
      onClose={handleClose}
      anchorEl={properties.anchorEl.current}
      anchorOrigin={
        properties.anchorOrigin || { vertical: "bottom", horizontal: "left" }
      }
      transformOrigin={
        properties.transformOrigin || { vertical: "top", horizontal: "left" }
      }
    >
      <ColorPickerDialog>
        <HexColorPicker
          color={color}
          onChange={(newColor): void => {
            setColor(newColor);
          }}
        />
        <HorizontalDivider />
        <ColorPickerInput>
          <HexWrapper>
            <HexLabel>{"Hex"}</HexLabel>
            <HexColorInputBox>
              <HashLabel>{"#"}</HashLabel>
              <HexColorInput
                color={color}
                onChange={(newColor): void => {
                  setColor(newColor);
                }}
              />
            </HexColorInputBox>
          </HexWrapper>
          <Swatch $color={color} />
        </ColorPickerInput>
        <HorizontalDivider />
        <ColorList>
          {presetColors.map((presetColor) => (
            <RecentColorButton
              key={presetColor}
              $color={presetColor}
              onClick={(): void => {
                setColor(presetColor);
              }}
            />
          ))}
        </ColorList>

        {recentColors.current.length > 0 ? (
          <>
            <HorizontalDivider />
            <RecentLabel>{"Recent"}</RecentLabel>
            <ColorList>
              {recentColors.current.map((recentColor) => (
                <RecentColorButton
                  key={recentColor}
                  $color={recentColor}
                  onClick={(): void => {
                    setColor(recentColor);
                  }}
                />
              ))}
            </ColorList>
          </>
        ) : (
          <div />
        )}
        <Buttons>
          <StyledButton
            onClick={(): void => {
              closePicker(color);
            }}
          >
            <Check
              style={{ width: "16px", height: "16px", marginRight: "8px" }}
            />
            {t("color_picker.apply")}
          </StyledButton>
        </Buttons>
      </ColorPickerDialog>
    </Popover>
  );
};

const Buttons = styled.div`
  display: flex;
  justify-content: flex-end;
  margin: 8px;
`;

const StyledButton = styled("div")`
  cursor: pointer;
  color: #ffffff;
  background: #f2994a;
  padding: 0px 10px;
  height: 36px;
  line-height: 36px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  font-family: "Inter";
  font-size: 14px;
  &:hover {
    background: #d68742;
  }
`;

const RecentLabel = styled.div`
  font-family: "Inter";
  font-size: 12px;
  font-family: Inter;
  margin: 8px 8px 0px 8px;
  color: ${theme.palette.text.secondary};
`;

const ColorList = styled.div`
  display: flex;
  flex-wrap: wrap;
  flex-direction: row;
  margin: 8px;
  justify-content: flex-start;
  gap: 4.7px;
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
    return $color;
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

const ColorPickerDialog = styled.div`
  background: ${theme.palette.background.default};
  width: ${colorPickerWidth}px;
  padding: 0px;
  display: flex;
  flex-direction: column;

  & .react-colorful {
    height: ${colorfulHeight}px;
    width: ${colorPickerWidth}px;
  }
  & .react-colorful__saturation {
    border-bottom: none;
    border-radius: 0px;
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
    border-bottom: 1px solid #eee;
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
  width: 140px;
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
  width: 28px;
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
