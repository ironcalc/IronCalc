import React, { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  BorderBottomIcon,
  BorderCenterHIcon,
  BorderCenterVIcon,
  BorderInnerIcon,
  BorderLeftIcon,
  BorderOuterIcon,
  BorderRightIcon,
  BorderTopIcon,
  BorderNoneIcon,
  BorderStyleIcon,
} from "../icons";
import ColorPicker from "./colorPicker";
import Popover, { PopoverOrigin } from "@mui/material/Popover";
import {
  Check,
  ChevronDown,
  Grid2X2 as BorderAllIcon,
  PencilLine,
  ChevronRight,
} from "lucide-react";
import { styled } from "@mui/material/styles";
import { theme } from "../theme";
import { BorderOptions, BorderStyle, BorderType } from "@ironcalc/wasm";

type BorderPickerProps = {
  className?: string;
  onChange: (border: BorderOptions) => void;
  anchorEl: React.RefObject<HTMLElement>;
  anchorOrigin?: PopoverOrigin;
  transformOrigin?: PopoverOrigin;
  open: boolean;
};

const BorderPicker = (properties: BorderPickerProps) => {
  const { t } = useTranslation();

  const [borderSelected, setBorderSelected] = useState(BorderType.None);
  const [borderColor, setBorderColor] = useState("#000000");
  const [borderStyle, setBorderStyle] = useState(BorderStyle.Thin);
  const [colorPickerOpen, setColorPickerOpen] = useState(false);
  const [stylePickerOpen, setStylePickerOpen] = useState(false);
  const closePicker = (): void => {
    properties.onChange({
      color: borderColor,
      style: borderStyle,
      border: borderSelected,
    });
  };
  const borderColorButton = useRef(null);
  const borderStyleButton = useRef(null);
  return (
    <>
      <StyledPopover
        open={properties.open}
        onClose={(): void => closePicker()}
        anchorEl={properties.anchorEl.current}
        anchorOrigin={
          properties.anchorOrigin || { vertical: "bottom", horizontal: "left" }
        }
        transformOrigin={
          properties.transformOrigin || { vertical: "top", horizontal: "left" }
        }
      >
        <BorderPickerDialog>
          <Borders>
            <Line>
              <Button
                type="button"
                $pressed={borderSelected === BorderType.All}
                onClick={() => {
                  if (borderSelected === BorderType.All) {
                    setBorderSelected(BorderType.None);
                  } else {
                    setBorderSelected(BorderType.All);
                  }
                }}
                disabled={false}
                title={t("workbook.toolbar.borders_button_title")}
              >
                <BorderAllIcon />
              </Button>
              <Button
                type="button"
                $pressed={borderSelected === BorderType.Inner}
                onClick={() => {
                  if (borderSelected === BorderType.Inner) {
                    setBorderSelected(BorderType.None);
                  } else {
                    setBorderSelected(BorderType.Inner);
                  }
                }}
                disabled={false}
                title={t("workbook.toolbar.borders_button_title")}
              >
                <BorderInnerIcon />
              </Button>
              <Button
                type="button"
                $pressed={borderSelected === BorderType.CenterH}
                onClick={() => {
                  if (borderSelected === BorderType.CenterH) {
                    setBorderSelected(BorderType.None);
                  } else {
                    setBorderSelected(BorderType.CenterH);
                  }
                }}
                disabled={false}
                title={t("workbook.toolbar.borders_button_title")}
              >
                <BorderCenterHIcon />
              </Button>
              <Button
                type="button"
                $pressed={borderSelected === BorderType.CenterV}
                onClick={() => {
                  if (borderSelected === BorderType.CenterV) {
                    setBorderSelected(BorderType.None);
                  } else {
                    setBorderSelected(BorderType.CenterV);
                  }
                }}
                disabled={false}
                title={t("workbook.toolbar.borders_button_title")}
              >
                <BorderCenterVIcon />
              </Button>
              <Button
                type="button"
                $pressed={borderSelected === BorderType.Outer}
                onClick={() => {
                  if (borderSelected === BorderType.Outer) {
                    setBorderSelected(BorderType.None);
                  } else {
                    setBorderSelected(BorderType.Outer);
                  }
                }}
                disabled={false}
                title={t("workbook.toolbar.borders_button_title")}
              >
                <BorderOuterIcon />
              </Button>
            </Line>
            <Line>
              <Button
                type="button"
                $pressed={borderSelected === BorderType.None}
                onClick={() => {
                  if (borderSelected === BorderType.None) {
                    setBorderSelected(BorderType.None);
                  } else {
                    setBorderSelected(BorderType.None);
                  }
                }}
                disabled={false}
                title={t("workbook.toolbar.borders_button_title")}
              >
                <BorderNoneIcon />
              </Button>
              <Button
                type="button"
                $pressed={borderSelected === BorderType.Top}
                onClick={() => {
                  if (borderSelected === BorderType.Top) {
                    setBorderSelected(BorderType.None);
                  } else {
                    setBorderSelected(BorderType.Top);
                  }
                }}
                disabled={false}
                title={t("workbook.toolbar.borders_button_title")}
              >
                <BorderTopIcon />
              </Button>
              <Button
                type="button"
                $pressed={borderSelected === BorderType.Right}
                onClick={() => {
                  if (borderSelected === BorderType.Right) {
                    setBorderSelected(BorderType.None);
                  } else {
                    setBorderSelected(BorderType.Right);
                  }
                }}
                disabled={false}
                title={t("workbook.toolbar.borders_button_title")}
              >
                <BorderRightIcon />
              </Button>
              <Button
                type="button"
                $pressed={borderSelected === BorderType.Bottom}
                onClick={() => {
                  if (borderSelected === BorderType.Bottom) {
                    setBorderSelected(BorderType.None);
                  } else {
                    setBorderSelected(BorderType.Bottom);
                  }
                }}
                disabled={false}
                title={t("workbook.toolbar.borders_button_title")}
              >
                <BorderBottomIcon />
              </Button>
              <Button
                type="button"
                $pressed={borderSelected === BorderType.Left}
                onClick={() => {
                  if (borderSelected === BorderType.Left) {
                    setBorderSelected(BorderType.None);
                  } else {
                    setBorderSelected(BorderType.Left);
                  }
                }}
                disabled={false}
                title={t("workbook.toolbar.borders_button_title")}
              >
                <BorderLeftIcon />
              </Button>
            </Line>
          </Borders>
          <Divider />
          <Styles>
            <ButtonWrapper onClick={() => setColorPickerOpen(true)}>
              <Button
                type="button"
                $pressed={false}
                disabled={false}
                ref={borderColorButton}
                title={t("workbook.toolbar.borders_button_title")}
              >
                <PencilLine />
              </Button>
              <div style={{flexGrow:2}}>Border color</div>
              <ChevronRightStyled />
            </ButtonWrapper>
            <ButtonWrapper onClick={() => setStylePickerOpen(true)} ref={borderStyleButton}>
              <Button
                type="button"
                $pressed={false}
                disabled={false}
                title={t("workbook.toolbar.borders_button_title")}
              >
                <BorderStyleIcon />
              </Button>
              <div style={{flexGrow:2}}>Border style</div>
              <ChevronRightStyled />
            </ButtonWrapper>
          </Styles>
        </BorderPickerDialog>
        <ColorPicker
          color={borderColor}
          onChange={(color): void => {
            setBorderColor(color);
            setColorPickerOpen(false);
          }}
          anchorEl={borderColorButton}
          open={colorPickerOpen}
        />
        <StyledPopover
          open={stylePickerOpen}
          onClose={(): void => {
            setStylePickerOpen(false);
          }}
          anchorEl={borderStyleButton.current}
          anchorOrigin={{ vertical: "bottom", horizontal: "right" }}
          transformOrigin={{ vertical: 38, horizontal: -6 }}
        >
          <BorderStyleDialog>
          <LineWrapper
              onClick={() => {
                setBorderStyle(BorderStyle.Dashed);
                setStylePickerOpen(false);
              }}
              $checked={borderStyle === BorderStyle.None}
            >
              <BorderDescription>None</BorderDescription>
              <NoneLine />
            </LineWrapper>
            <LineWrapper
              onClick={() => {
                setBorderStyle(BorderStyle.Thin);
                setStylePickerOpen(false);
              }}
              $checked={borderStyle === BorderStyle.Thin}
            >
              <BorderDescription>Thin</BorderDescription>
              <SolidLine />
            </LineWrapper>
            <LineWrapper
              onClick={() => {
                setBorderStyle(BorderStyle.Medium);
                setStylePickerOpen(false);
              }}
              $checked={borderStyle === BorderStyle.Medium}
            >
              <BorderDescription>Medium</BorderDescription>
              <MediumLine />
            </LineWrapper>
            <LineWrapper
              onClick={() => {
                setBorderStyle(BorderStyle.Thick);
                setStylePickerOpen(false);
              }}
              $checked={borderStyle === BorderStyle.Thick}
            >
              <BorderDescription>Thick</BorderDescription>
              <ThickLine />
            </LineWrapper>
            <LineWrapper
              onClick={() => {
                setBorderStyle(BorderStyle.Dotted);
                setStylePickerOpen(false);
              }}
              $checked={borderStyle === BorderStyle.Dotted}
            >
              <BorderDescription>Dotted</BorderDescription>
              <DottedLine />
            </LineWrapper>
            <LineWrapper
              onClick={() => {
                setBorderStyle(BorderStyle.Dashed);
                setStylePickerOpen(false);
              }}
              $checked={borderStyle === BorderStyle.Dashed}
            >
              <BorderDescription>Dashed</BorderDescription>
              <DashedLine />
            </LineWrapper>
            <LineWrapper
              onClick={() => {
                setBorderStyle(BorderStyle.Dashed);
                setStylePickerOpen(false);
              }}
              $checked={borderStyle === BorderStyle.Double}
            >
              <BorderDescription>Double</BorderDescription>
              <DoubleLine />
            </LineWrapper>
          </BorderStyleDialog>
        </StyledPopover>
      </StyledPopover>
    </>
  );
};

type LineWrapperProperties = { $checked: boolean };
const LineWrapper = styled("div")<LineWrapperProperties>`
  display: flex;
  flex-direction: row;
  align-items: center;
  background-color: ${({ $checked }): string => {
    if ($checked) {
      return '#EEEEEE;';
    } else {
      return 'inherit;';
    }
  }};
  &:hover {
    border: 1px solid #EEEEEE;
  }
  padding:8px;
  cursor: pointer;
  border-radius: 4px;
  border: 1px solid white;
`;

const CheckIconWrapper = styled("div")`
  width: 12px;
`;

type CheckIconProperties = { $checked: boolean };
const CheckIcon = styled("div")<CheckIconProperties>`
  width: 2px;
  background-color: #EEE;
  height: 28px;
  visibility: ${({ $checked }): string => {
    if ($checked) {
      return "visible";
    }
    return "hidden";
  }};
`;
const NoneLine = styled("div")`
  width: 68px;
  border-top: 1px solid #E0E0E0;
`;
const SolidLine = styled("div")`
  width: 68px;
  border-top: 1px solid #333333;
`;
const MediumLine = styled("div")`
  width: 68px;
  border-top: 2px solid #333333;
`;
const ThickLine = styled("div")`
  width: 68px;
  border-top: 3px solid #333333;
`;
const DashedLine = styled("div")`
  width: 68px;
  border-top: 1px dashed #333333;
`;
const DottedLine = styled("div")`
  width: 68px;
  border-top: 1px dotted #333333;
`;
const DoubleLine = styled('div')`
  width: 68px;
  border-top: 3px double #333333;
`;

const Divider = styled("div")`
  display: inline-flex;
  heigh: 1px;
  border-bottom: 1px solid #EEE;
  margin-left: 0px;
  margin-right: 0px;
`;

const Borders = styled("div")`
  display: flex;
  flex-direction: column;
  padding-bottom: 4px;
`;

const Styles = styled("div")`
  display: flex;
  flex-direction: column;
`;

const Line = styled("div")`
  display: flex;
  flex-direction: row;
  align-items: center;
`;

const ButtonWrapper = styled("div")`
  display: flex;
  flex-direction: row;
  align-items: center;
  &:hover {
    background-color: #EEE;
    border-top-color: ${(): string => theme.palette.grey["400"]};
  }
  cursor: pointer;
  padding: 8px
`;

const BorderStyleDialog = styled("div")`
  background: ${({ theme }): string => theme.palette.background.default};
  padding: 4px;
  display: flex;
  flex-direction: column;
  align-items: center;
`;

const StyledPopover = styled(Popover)`
  .MuiPopover-paper {
    border-radius: 10px;
    border: 0px solid ${({ theme }): string => theme.palette.background.default};
    box-shadow: 1px 2px 8px rgba(139, 143, 173, 0.5);
  }
  .MuiPopover-padding {
    padding: 0px;
  }
  .MuiList-padding {
    padding: 0px;
  }
  font-family: ${({ theme }) => theme.typography.fontFamily};
  font-size: 13px;
`;

const BorderPickerDialog = styled("div")`
  background: ${({ theme }): string => theme.palette.background.default};
  padding: 4px;
  display: flex;
  flex-direction: column;
`;

const BorderDescription = styled("div")`
width: 70px;
`;

// type TypeButtonProperties = { $pressed: boolean; $underlinedColor?: string };
// const Button = styled.button<TypeButtonProperties>`
//   width: 23px;
//   height: 23px;
//   display: inline-flex;
//   align-items: center;
//   justify-content: center;
//   font-size: 14px;
//   border-radius: 2px;
//   margin-right: 5px;
//   transition: all 0.2s;

//   ${({ theme, disabled, $pressed, $underlinedColor }): string => {
//     if (disabled) {
//       return `
//       color: ${theme.palette.grey['600']};
//       cursor: default;
//     `;
//     }
//     return `
//       border-top: ${$underlinedColor ? '3px solid #FFF' : 'none'};
//       border-bottom: ${$underlinedColor ? `3px solid ${$underlinedColor}` : 'none'};
//       color: ${theme.palette.text.primary};
//       background-color: ${$pressed ? theme.palette.grey['600'] : '#FFF'};
//       &:hover {
//         background-color: ${theme.palette.grey['400']};
//         border-top-color: ${theme.palette.grey['400']};
//       }
//     `;
//   }}
// `;

type TypeButtonProperties = { $pressed: boolean; $underlinedColor?: string };
const Button = styled("button")<TypeButtonProperties>(
  ({ disabled, $pressed, $underlinedColor }) => {
    let result: Record<string, any> = {
      width: "24px",
      height: "24px",
      display: "inline-flex",
      alignItems: "center",
      justifyContent: "center",
      // fontSize: "26px",
      border: "0px solid #fff",
      borderRadius: "2px",
      marginRight: "5px",
      transition: "all 0.2s",
      cursor: "pointer",
      padding: "0px",
    };
    if (disabled) {
      result.color = theme.palette.grey["600"];
      result.cursor = "default";
    } else {
      result.borderTop = $underlinedColor ? "3px solid #FFF" : "none";
      result.borderBottom = $underlinedColor
        ? `3px solid ${$underlinedColor}`
        : "none";
      (result.color = "#21243A"),
        (result.backgroundColor = $pressed
          ? theme.palette.grey["600"]
          : "inherit");
      result["&:hover"] = {
        backgroundColor: "#F1F2F8",
        borderTopColor: "#F1F2F8",
      };
      result["svg"] = {
        width: "16px",
        height: "16px",
      };
    }
    return result;
  }
);

const ChevronRightStyled = styled(ChevronRight)`
  width: 16px;
  height: 16px;
`;

export default BorderPicker;
