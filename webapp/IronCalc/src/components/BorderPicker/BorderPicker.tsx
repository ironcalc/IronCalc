import { type BorderOptions, BorderStyle, BorderType } from "@ironcalc/wasm";
import ClickAwayListener from "@mui/material/ClickAwayListener";
import MenuItem from "@mui/material/MenuItem";
import Popper, { type PopperPlacementType } from "@mui/material/Popper";
import { styled } from "@mui/material/styles";
import {
  Grid2X2 as BorderAllIcon,
  ChevronRight,
  PencilLine,
} from "lucide-react";
import type React from "react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  BorderBottomIcon,
  BorderCenterHIcon,
  BorderCenterVIcon,
  BorderInnerIcon,
  BorderLeftIcon,
  BorderNoneIcon,
  BorderOuterIcon,
  BorderRightIcon,
  BorderStyleIcon,
  BorderTopIcon,
} from "../../icons";
import { theme } from "../../theme";
import ColorPicker from "../ColorPicker/ColorPicker";

type BorderPickerProps = {
  className?: string;
  onChange: (border: BorderOptions) => void;
  onClose: () => void;
  anchorEl: React.RefObject<HTMLElement | null>;
  placement: PopperPlacementType;
  open: boolean;
};

const BorderPicker = (properties: BorderPickerProps) => {
  const { t } = useTranslation();

  const [borderSelected, setBorderSelected] = useState<BorderType | null>(null);
  const [borderColor, setBorderColor] = useState(theme.palette.common.white);
  const [borderStyle, setBorderStyle] = useState(BorderStyle.Thin);
  const [colorPickerOpen, setColorPickerOpen] = useState(false);
  const [stylePickerOpen, setStylePickerOpen] = useState(false);

  // FIXME
  // biome-ignore lint/correctness/useExhaustiveDependencies: We don't want updating the function every time the properties.onChange
  useEffect(() => {
    if (!borderSelected) {
      return;
    }
    properties.onChange({
      color: borderColor,
      style: borderStyle,
      border: borderSelected,
    });
  }, [borderColor, borderStyle, borderSelected]);

  const onClose = properties.onClose;

  // The reason is that the border picker doesn't start with the properties of the selected area
  // biome-ignore lint/correctness/useExhaustiveDependencies: We reset the styles, every time we open (or close) the widget
  useEffect(() => {
    setBorderSelected(null);
    setBorderColor(theme.palette.common.black);
    setBorderStyle(BorderStyle.Thin);
  }, [properties.open]);

  const borderColorButton = useRef(null);
  const borderStyleButton = useRef(null);
  const stylePickerCloseTimeout = useRef<NodeJS.Timeout | null>(null);

  const handleStylePickerOpen = () => {
    if (stylePickerCloseTimeout.current) {
      clearTimeout(stylePickerCloseTimeout.current);
    }
    setStylePickerOpen(true);
  };

  const handleStylePickerClose = () => {
    if (stylePickerCloseTimeout.current) {
      clearTimeout(stylePickerCloseTimeout.current);
    }
    stylePickerCloseTimeout.current = setTimeout(() => {
      setStylePickerOpen(false);
    }, 150);
  };

  useEffect(
    () => () => {
      if (stylePickerCloseTimeout.current) {
        clearTimeout(stylePickerCloseTimeout.current);
      }
    },
    [],
  );

  if (!properties.anchorEl.current) {
    return null;
  }

  return (
    <StyledPopper
      open={properties.open}
      anchorEl={properties.anchorEl.current}
      placement={properties.placement || "bottom-start"}
      keepMounted={false}
      modifiers={[
        {
          name: "offset",
          options: {
            offset: [-4, 4],
          },
        },
      ]}
    >
      <ClickAwayListener onClickAway={onClose}>
        <PopperContent>
          <BorderPickerDialog>
            <Borders>
              <Line>
                <Button
                  type="button"
                  $pressed={borderSelected === BorderType.All}
                  onClick={() => {
                    if (borderSelected === BorderType.All) {
                      setBorderSelected(null);
                    } else {
                      setBorderSelected(BorderType.All);
                    }
                  }}
                  disabled={false}
                  title={t("toolbar.borders.all")}
                >
                  <BorderAllIcon />
                </Button>
                <Button
                  type="button"
                  $pressed={borderSelected === BorderType.Inner}
                  onClick={() => {
                    if (borderSelected === BorderType.Inner) {
                      setBorderSelected(null);
                    } else {
                      setBorderSelected(BorderType.Inner);
                    }
                  }}
                  disabled={false}
                  title={t("toolbar.borders.inner")}
                >
                  <BorderInnerIcon />
                </Button>
                <Button
                  type="button"
                  $pressed={borderSelected === BorderType.CenterH}
                  onClick={() => {
                    if (borderSelected === BorderType.CenterH) {
                      setBorderSelected(null);
                    } else {
                      setBorderSelected(BorderType.CenterH);
                    }
                  }}
                  disabled={false}
                  title={t("toolbar.borders.horizontal")}
                >
                  <BorderCenterHIcon />
                </Button>
                <Button
                  type="button"
                  $pressed={borderSelected === BorderType.CenterV}
                  onClick={() => {
                    if (borderSelected === BorderType.CenterV) {
                      setBorderSelected(null);
                    } else {
                      setBorderSelected(BorderType.CenterV);
                    }
                  }}
                  disabled={false}
                  title={t("toolbar.borders.vertical")}
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
                  title={t("toolbar.borders.outer")}
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
                  title={t("toolbar.borders.clear")}
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
                  title={t("toolbar.borders.top")}
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
                  title={t("toolbar.borders.right")}
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
                  title={t("toolbar.borders.bottom")}
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
                  title={t("toolbar.borders.left")}
                >
                  <BorderLeftIcon />
                </Button>
              </Line>
            </Borders>
            <Divider />
            <Styles>
              <MenuItemWrapper
                onClick={() => setColorPickerOpen(true)}
                ref={borderColorButton}
              >
                <PencilLine />
                <MenuItemText>Border color</MenuItemText>
                <ChevronRightStyled />
              </MenuItemWrapper>

              <MenuItemWrapper
                onMouseEnter={handleStylePickerOpen}
                onMouseLeave={handleStylePickerClose}
                ref={borderStyleButton}
              >
                <BorderStyleIcon />
                <MenuItemText>Border style</MenuItemText>
                <ChevronRightStyled />
              </MenuItemWrapper>
            </Styles>
          </BorderPickerDialog>
          <ColorPicker
            color={borderColor}
            defaultColor={theme.palette.common.black}
            title={t("color_picker.default")}
            onChange={(color): void => {
              setBorderColor(color);
              setColorPickerOpen(false);
            }}
            onClose={() => {
              setColorPickerOpen(false);
            }}
            anchorEl={borderColorButton}
            open={colorPickerOpen}
            anchorOrigin={{
              vertical: "top",
              horizontal: "right",
            }}
            transformOrigin={{
              vertical: "top",
              horizontal: "left",
            }}
          />
          {borderStyleButton.current && (
            <StyledPopper
              open={stylePickerOpen}
              anchorEl={borderStyleButton.current}
              placement="right-start"
              keepMounted={false}
              modifiers={[
                {
                  name: "offset",
                  options: {
                    offset: [-4, 0],
                  },
                },
              ]}
            >
              <StylePicker
                onMouseEnter={handleStylePickerOpen}
                onMouseLeave={handleStylePickerClose}
              >
                <StyledMenuItem
                  onClick={() => {
                    setBorderStyle(BorderStyle.Thin);
                    setStylePickerOpen(false);
                  }}
                  selected={borderStyle === BorderStyle.Thin}
                >
                  <SolidLine />
                </StyledMenuItem>
                <StyledMenuItem
                  onClick={() => {
                    setBorderStyle(BorderStyle.Medium);
                    setStylePickerOpen(false);
                  }}
                  selected={borderStyle === BorderStyle.Medium}
                >
                  <MediumLine />
                </StyledMenuItem>
                <StyledMenuItem
                  onClick={() => {
                    setBorderStyle(BorderStyle.Thick);
                    setStylePickerOpen(false);
                  }}
                  selected={borderStyle === BorderStyle.Thick}
                >
                  <ThickLine />
                </StyledMenuItem>
                <StyledMenuItem
                  onClick={() => {
                    setBorderStyle(BorderStyle.Double);
                    setStylePickerOpen(false);
                  }}
                  selected={borderStyle === BorderStyle.Double}
                >
                  <DoubleLine />
                </StyledMenuItem>
                <StyledMenuItem
                  onClick={() => {
                    setBorderStyle(BorderStyle.Dotted);
                    setStylePickerOpen(false);
                  }}
                  selected={borderStyle === BorderStyle.Dotted}
                >
                  <DottedLine />
                </StyledMenuItem>
                <StyledMenuItem
                  onClick={() => {
                    setBorderStyle(BorderStyle.MediumDashed);
                    setStylePickerOpen(false);
                  }}
                  selected={borderStyle === BorderStyle.MediumDashed}
                >
                  <MediumDashedLine />
                </StyledMenuItem>
              </StylePicker>
            </StyledPopper>
          )}
        </PopperContent>
      </ClickAwayListener>
    </StyledPopper>
  );
};

const StyledMenuItem = styled(MenuItem)`
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: center;
  height: 32px;
  padding: 8px;
  border-radius: 4px;
  &::before {
    content: none;
  }
  &.Mui-selected {
    background-color: ${({ theme }) => theme.palette.action.hover};
    &:hover {
      background-color: ${({ theme }) => theme.palette.action.hover};
    }
  }
`;

const SolidLine = styled("div")`
  width: 68px;
  border-top: 1px solid ${theme.palette.grey["900"]};
`;
const MediumLine = styled("div")`
  width: 68px;
  border-top: 2px solid ${theme.palette.grey["900"]};
`;
const ThickLine = styled("div")`
  width: 68px;
  border-top: 3px solid ${theme.palette.grey["900"]};
`;

const DoubleLine = styled("div")`
  width: 68px;
  height: 3px;
  position: relative;
  &::before {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    border-top: 1px solid ${theme.palette.grey["900"]};
  }
  &::after {
    content: "";
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    border-top: 1px solid ${theme.palette.grey["900"]};
  }
`;

const DottedLine = styled("div")`
  width: 68px;
  border-top: 1px dotted ${theme.palette.grey["900"]};
`;

const MediumDashedLine = styled("div")`
  width: 68px;
  border-top: 2px dashed ${theme.palette.grey["900"]};
`;

const Divider = styled("div")`
  width: 100%;
  margin: auto;
  border-top: 1px solid ${theme.palette.grey["200"]};
`;

const Borders = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 4px;
`;

const Styles = styled("div")`
  display: flex;
  flex-direction: column;
  padding: 4px;
`;

const Line = styled("div")`
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 4px;
`;

const BaseMenuItem = (props: React.ComponentProps<typeof MenuItem>) => (
  <MenuItem disableRipple {...props} />
);

const MenuItemWrapper = styled(BaseMenuItem)`
  display: flex;
  justify-content: flex-start;
  border-radius: 4px;
  padding: 8px;
  height: 32px;
  min-height: 32px;
  max-height: 32px;
  color: ${theme.palette.common.black};
  font-size: 12px;
  gap: 8px;
  svg {
    max-width: 16px;
    min-width: 16px;
    max-height: 16px;
    min-height: 16px;
    color: ${theme.palette.grey[600]};
  }
`;

const MenuItemText = styled("div")`
  flex-grow: 1;
`;

const PopperContent = styled("div")`
  border-radius: 8px;
  border: 0px solid ${({ theme }): string => theme.palette.background.default};
  box-shadow: 1px 2px 8px rgba(139, 143, 173, 0.5);
  background: ${({ theme }): string => theme.palette.background.default};
  font-family: ${({ theme }) => theme.typography.fontFamily};
  font-size: 12px;
  overflow: hidden;
`;

const StylePicker = styled(PopperContent)`
  padding: 4px;
  display: flex;
  flex-direction: column;
  align-items: center;
`;

const StyledPopper = styled(Popper)`
  z-index: 1300;
  &[data-popper-placement] {
    pointer-events: auto;
  }
`;

const BorderPickerDialog = styled("div")`
  background: ${({ theme }): string => theme.palette.background.default};
  display: flex;
  flex-direction: column;
`;

type TypeButtonProperties = { $pressed: boolean; $underlinedColor?: string };
const Button = styled("button")<TypeButtonProperties>(
  ({ disabled, $pressed, $underlinedColor }) => {
    const result = {
      width: "24px",
      height: "24px",
      display: "inline-flex",
      alignItems: "center",
      justifyContent: "center",
      // fontSize: "26px",
      border: `0px solid ${theme.palette.common.white}`,
      borderRadius: "4px",
      cursor: "pointer",
      padding: "0px",
    };
    if (disabled) {
      return {
        ...result,
        color: theme.palette.grey["600"],
        cursor: "default",
      };
    }
    return {
      ...result,
      borderTop: $underlinedColor
        ? `3px solid ${theme.palette.common.white}`
        : "none",
      borderBottom: $underlinedColor ? `3px solid ${$underlinedColor}` : "none",
      color: `${theme.palette.grey["900"]}`,
      backgroundColor: $pressed ? theme.palette.grey["200"] : "inherit",
      "&:hover": {
        outline: `1px solid ${theme.palette.grey["200"]}`,
        borderTopColor: theme.palette.grey["200"],
      },
      svg: {
        width: "16px",
        height: "16px",
      },
    };
  },
);

const ChevronRightStyled = styled(ChevronRight)`
  width: 16px;
  height: 16px;
`;

export default BorderPicker;
