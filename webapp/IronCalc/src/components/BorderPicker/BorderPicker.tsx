import { type BorderOptions, BorderStyle, BorderType } from "@ironcalc/wasm";
import ClickAwayListener from "@mui/material/ClickAwayListener";
import MenuItem from "@mui/material/MenuItem";
import Popper, { type PopperPlacementType } from "@mui/material/Popper";
import { styled, useTheme } from "@mui/material/styles";
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
import { IconButton } from "../Button/IconButton";
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
  const theme = useTheme();

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
  const stylePickerCloseTimeout = useRef<ReturnType<typeof setTimeout> | null>(
    null,
  );

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
      placement={properties.placement}
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
              <Row>
                <IconButton
                  pressed={borderSelected === BorderType.All}
                  aria-label={t("toolbar.borders.all")}
                  title={t("toolbar.borders.all")}
                  icon={<BorderAllIcon />}
                  onClick={() => {
                    if (borderSelected === BorderType.All) {
                      setBorderSelected(null);
                    } else {
                      setBorderSelected(BorderType.All);
                    }
                  }}
                />
                <IconButton
                  pressed={borderSelected === BorderType.Inner}
                  aria-label={t("toolbar.borders.inner")}
                  title={t("toolbar.borders.inner")}
                  icon={<BorderInnerIcon />}
                  onClick={() => {
                    if (borderSelected === BorderType.Inner) {
                      setBorderSelected(null);
                    } else {
                      setBorderSelected(BorderType.Inner);
                    }
                  }}
                />
                <IconButton
                  pressed={borderSelected === BorderType.CenterH}
                  aria-label={t("toolbar.borders.horizontal")}
                  title={t("toolbar.borders.horizontal")}
                  icon={<BorderCenterHIcon />}
                  onClick={() => {
                    if (borderSelected === BorderType.CenterH) {
                      setBorderSelected(null);
                    } else {
                      setBorderSelected(BorderType.CenterH);
                    }
                  }}
                />
                <IconButton
                  pressed={borderSelected === BorderType.CenterV}
                  aria-label={t("toolbar.borders.vertical")}
                  title={t("toolbar.borders.vertical")}
                  icon={<BorderCenterVIcon />}
                  onClick={() => {
                    if (borderSelected === BorderType.CenterV) {
                      setBorderSelected(null);
                    } else {
                      setBorderSelected(BorderType.CenterV);
                    }
                  }}
                />
                <IconButton
                  pressed={borderSelected === BorderType.Outer}
                  aria-label={t("toolbar.borders.outer")}
                  title={t("toolbar.borders.outer")}
                  icon={<BorderOuterIcon />}
                  onClick={() => {
                    if (borderSelected === BorderType.Outer) {
                      setBorderSelected(BorderType.None);
                    } else {
                      setBorderSelected(BorderType.Outer);
                    }
                  }}
                />
              </Row>
              <Row>
                <IconButton
                  pressed={borderSelected === BorderType.None}
                  aria-label={t("toolbar.borders.clear")}
                  title={t("toolbar.borders.clear")}
                  icon={<BorderNoneIcon />}
                  onClick={() => {
                    setBorderSelected(BorderType.None);
                  }}
                />
                <IconButton
                  pressed={borderSelected === BorderType.Top}
                  aria-label={t("toolbar.borders.top")}
                  title={t("toolbar.borders.top")}
                  icon={<BorderTopIcon />}
                  onClick={() => {
                    if (borderSelected === BorderType.Top) {
                      setBorderSelected(BorderType.None);
                    } else {
                      setBorderSelected(BorderType.Top);
                    }
                  }}
                />
                <IconButton
                  pressed={borderSelected === BorderType.Right}
                  aria-label={t("toolbar.borders.right")}
                  title={t("toolbar.borders.right")}
                  icon={<BorderRightIcon />}
                  onClick={() => {
                    if (borderSelected === BorderType.Right) {
                      setBorderSelected(BorderType.None);
                    } else {
                      setBorderSelected(BorderType.Right);
                    }
                  }}
                />
                <IconButton
                  pressed={borderSelected === BorderType.Bottom}
                  aria-label={t("toolbar.borders.bottom")}
                  title={t("toolbar.borders.bottom")}
                  icon={<BorderBottomIcon />}
                  onClick={() => {
                    if (borderSelected === BorderType.Bottom) {
                      setBorderSelected(BorderType.None);
                    } else {
                      setBorderSelected(BorderType.Bottom);
                    }
                  }}
                />
                <IconButton
                  pressed={borderSelected === BorderType.Left}
                  aria-label={t("toolbar.borders.left")}
                  title={t("toolbar.borders.left")}
                  icon={<BorderLeftIcon />}
                  onClick={() => {
                    if (borderSelected === BorderType.Left) {
                      setBorderSelected(BorderType.None);
                    } else {
                      setBorderSelected(BorderType.Left);
                    }
                  }}
                />
              </Row>
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
                  <SolidLine thickness={1} />
                </StyledMenuItem>
                <StyledMenuItem
                  onClick={() => {
                    setBorderStyle(BorderStyle.Medium);
                    setStylePickerOpen(false);
                  }}
                  selected={borderStyle === BorderStyle.Medium}
                >
                  <SolidLine thickness={2} />
                </StyledMenuItem>
                <StyledMenuItem
                  onClick={() => {
                    setBorderStyle(BorderStyle.Thick);
                    setStylePickerOpen(false);
                  }}
                  selected={borderStyle === BorderStyle.Thick}
                >
                  <SolidLine thickness={3} />
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
                <StyledMenuItem
                  onClick={() => {
                    setBorderStyle(BorderStyle.SlantDashDot);
                    setStylePickerOpen(false);
                  }}
                  selected={borderStyle === BorderStyle.SlantDashDot}
                >
                  <SlantDashDotLine />
                </StyledMenuItem>
                <StyledMenuItem
                  onClick={() => {
                    setBorderStyle(BorderStyle.MediumDashDot);
                    setStylePickerOpen(false);
                  }}
                  selected={borderStyle === BorderStyle.MediumDashDot}
                >
                  <MediumDashDotLine />
                </StyledMenuItem>
                <StyledMenuItem
                  onClick={() => {
                    setBorderStyle(BorderStyle.MediumDashDotDot);
                    setStylePickerOpen(false);
                  }}
                  selected={borderStyle === BorderStyle.MediumDashDotDot}
                >
                  <MediumDashDotDotLine />
                </StyledMenuItem>
              </StylePicker>
            </StyledPopper>
          )}
        </PopperContent>
      </ClickAwayListener>
    </StyledPopper>
  );
};

const borderLinePreviewWidth = 68;

const dashDotGradient = (color: string) =>
  `repeating-linear-gradient(90deg, ${color} 0px 4px, transparent 4px 6px, ${color} 6px 7px, transparent 7px 9px)`;

const dashDotDotGradient = (color: string) =>
  `repeating-linear-gradient(90deg, ${color} 0px 4px, transparent 4px 6px, ${color} 6px 7px, transparent 7px 9px, ${color} 9px 10px, transparent 10px 12px)`;

const StyledMenuItem = styled(MenuItem)(({ theme }) => ({
  display: "flex",
  flexDirection: "row",
  alignItems: "center",
  justifyContent: "center",
  height: 32,
  padding: 8,
  borderRadius: 4,

  "&::before": {
    content: "none",
  },

  "&.Mui-selected": {
    backgroundColor: theme.palette.action.hover,
    "&:hover": {
      backgroundColor: theme.palette.action.hover,
    },
  },
}));

type SolidLineProps = { thickness: 1 | 2 | 3 };

const SolidLine = styled("div")<SolidLineProps>(({ theme, thickness }) => ({
  width: borderLinePreviewWidth,
  borderTop: `${thickness}px solid ${theme.palette.grey[900]}`,
}));

const DoubleLine = styled("div")(({ theme }) => ({
  width: borderLinePreviewWidth,
  height: 3,
  position: "relative",

  "&::before": {
    content: '""',
    position: "absolute",
    top: 0,
    left: 0,
    right: 0,
    borderTop: `1px solid ${theme.palette.grey[900]}`,
  },

  "&::after": {
    content: '""',
    position: "absolute",
    bottom: 0,
    left: 0,
    right: 0,
    borderTop: `1px solid ${theme.palette.grey[900]}`,
  },
}));

const DottedLine = styled("div")(({ theme }) => ({
  width: `${borderLinePreviewWidth}px`,
  borderTop: `1px dotted ${theme.palette.grey[900]}`,
}));

const MediumDashedLine = styled("div")(({ theme }) => ({
  width: borderLinePreviewWidth,
  borderTop: `2px dashed ${theme.palette.grey[900]}`,
}));

const SlantDashDotLine = styled("div")(({ theme }) => ({
  width: borderLinePreviewWidth,
  height: 1,
  background: dashDotGradient(theme.palette.grey[900]),
}));

const MediumDashDotLine = styled("div")(({ theme }) => ({
  width: borderLinePreviewWidth,
  height: 2,
  background: dashDotGradient(theme.palette.grey[900]),
}));

const MediumDashDotDotLine = styled("div")(({ theme }) => ({
  width: borderLinePreviewWidth,
  height: 2,
  background: dashDotDotGradient(theme.palette.grey[900]),
}));

const Divider = styled("div")(({ theme }) => ({
  width: "100%",
  margin: "auto",
  borderTop: `1px solid ${theme.palette.grey["200"]}`,
}));

const Borders = styled("div")({
  display: "flex",
  flexDirection: "column",
  gap: 4,
  padding: 4,
});

const Styles = styled("div")({
  display: "flex",
  flexDirection: "column",
  padding: 4,
});

const Row = styled("div")({
  display: "flex",
  flexDirection: "row",
  alignItems: "center",
  gap: 2,
});

const BaseMenuItem = (props: React.ComponentProps<typeof MenuItem>) => (
  <MenuItem disableRipple {...props} />
);

const MenuItemWrapper = styled(BaseMenuItem)(({ theme }) => ({
  display: "flex",
  justifyContent: "flex-start",
  borderRadius: 4,
  padding: 8,
  height: 32,
  minHeight: 32,
  maxHeight: 32,
  color: theme.palette.common.black,
  fontSize: 12,
  gap: 8,
  svg: {
    maxWidth: 16,
    minWidth: 16,
    maxHeight: 16,
    minHeight: 16,
    color: theme.palette.grey[600],
  },
}));

const MenuItemText = styled("div")({
  flexGrow: 1,
});

const PopperContent = styled("div")(({ theme }) => ({
  borderRadius: 8,
  border: `0px solid ${theme.palette.background.default}`,
  boxShadow: "1px 2px 8px rgba(139, 143, 173, 0.5)",
  background: theme.palette.background.default,
  fontFamily: theme.typography.fontFamily,
  fontSize: 12,
  overflow: "hidden",
}));

const StylePicker = styled(PopperContent)({
  padding: 4,
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
});

const StyledPopper = styled(Popper)({
  zIndex: 1300,
  "&[data-popper-placement]": {
    pointerEvents: "auto",
  },
});

const BorderPickerDialog = styled("div")(({ theme }) => ({
  background: theme.palette.background.default,
  display: "flex",
  flexDirection: "column",
}));

const ChevronRightStyled = styled(ChevronRight)({
  width: 16,
  height: 16,
});

export default BorderPicker;
