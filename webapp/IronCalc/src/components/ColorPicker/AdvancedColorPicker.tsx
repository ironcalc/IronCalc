import { Popover, type PopoverOrigin, styled } from "@mui/material";
import { Check } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { HexColorInput, HexColorPicker } from "react-colorful";
import { useTranslation } from "react-i18next";
import { Button } from "../Button/Button";

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
        <ButtonsWrapper>
          <Button size="sm" variant="secondary" onClick={onCancel}>
            {t("color_picker.cancel")}
          </Button>
          <Button
            size="sm"
            startIcon={<Check />}
            onClick={(): void => {
              handleColorSelect(selectedColor);
              onCancel();
            }}
          >
            {t("color_picker.apply")}
          </Button>
        </ButtonsWrapper>
      </ColorPickerDialog>
    </StylePopover>
  );
};

const StylePopover = styled(Popover)({
  "& .MuiPaper-root": {
    borderRadius: 8,
    padding: 0,
    marginLeft: -4,
    maxWidth: 220,
  },
});

const HorizontalDivider = styled("div")(({ theme }) => ({
  height: 0,
  width: "100%",
  borderTop: `1px solid ${theme.palette.grey[200]}`,
}));

// Color Picker Dialog Styles
const ColorPickerDialog = styled("div")(({ theme }) => ({
  background: theme.palette.background.default,
  width: 240,
  padding: 0,
  display: "flex",
  flexDirection: "column",
  maxWidth: "100%",

  "& .react-colorful": {
    height: 160,
    width: "100%",
  },
  "& .react-colorful__saturation": {
    borderBottom: "none",
    borderRadius: "8px 8px 0px 0px",
  },
  "& .react-colorful__hue": {
    height: 8,
    margin: 8,
    borderRadius: 5,
  },
  "& .react-colorful__saturation-pointer": {
    width: 14,
    height: 14,
  },
  "& .react-colorful__hue-pointer": {
    borderRadius: 8,
    height: 16,
    width: 16,
  },
}));

const ButtonsWrapper = styled("div")({
  display: "flex",
  justifyContent: "space-between",
  margin: 8,
  gap: 8,
});

const HashLabel = styled("div")(({ theme }) => ({
  margin: "auto 0px auto 10px",
  fontSize: 13,
  color: "#333",
  fontFamily: theme.typography.button.fontFamily,
}));

const HexLabel = styled("div")(({ theme }) => ({
  margin: "auto 0px",
  fontSize: 12,
  display: "inline-flex",
  fontFamily: theme.typography.button.fontFamily,
}));

const HexColorInputBox = styled("div")(({ theme }) => ({
  display: "inline-flex",
  flexGrow: 1,
  width: "100%",
  height: 28,
  border: `1px solid ${theme.palette.grey[300]}`,
  borderRadius: 5,

  "&:hover": {
    border: `1px solid ${theme.palette.grey[600]}`,
  },

  "&:focus-within": {
    outline: `2px solid ${theme.palette.secondary.main}`,
    outlineOffset: 1,
  },
}));

const StyledHexColorInput = styled(HexColorInput)(({ theme }) => ({
  width: "100%",
  border: "none",
  background: "transparent",
  outline: "none",
  fontFamily: theme.typography.button.fontFamily,
  fontSize: 12,
  textTransform: "uppercase",
  textAlign: "right",
  paddingRight: 10,
  borderRadius: 5,

  "&:focus": {
    borderColor: "#4298ef",
  },
}));

const HexWrapper = styled("div")(({ theme }) => ({
  display: "flex",
  gap: 8,
  flexGrow: 1,

  "& input": {
    minWidth: 0,
    border: 0,
    background: theme.palette.background.default,
    outline: "none",
    fontFamily: theme.typography.button.fontFamily,
    fontSize: 12,
    textTransform: "uppercase",
    textAlign: "right",
    paddingRight: 10,
    borderRadius: 5,
  },

  "& input:focus": {
    borderColor: "#4298ef",
  },
}));

const Swatch = styled("div")<{ $color: string }>(({ $color, theme }) => ({
  display: "inline-flex",
  border:
    $color.toUpperCase() === "#FFFFFF"
      ? `1px solid ${theme.palette.grey[300]}`
      : `1px solid ${$color}`,
  backgroundColor: $color,
  minWidth: 28,
  height: 28,
  borderRadius: 5,
}));

const ColorPickerInput = styled("div")({
  display: "flex",
  flexDirection: "row",
  alignItems: "center",
  margin: 8,
  gap: 8,
});

export default AdvancedColorPicker;
