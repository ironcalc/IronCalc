import {
  Menu,
  MenuItem,
  type PopoverOrigin,
  styled,
  useTheme,
} from "@mui/material";
import { Check, Plus } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
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

  const theme = useTheme();

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

  const renderColorSwatch = (presetColor: string) => {
    const isSelected =
      selectedColor.toUpperCase() === presetColor.toUpperCase();
    return (
      <SelectableColorSwatch
        key={presetColor}
        $color={presetColor}
        onClick={() => handleColorSelect(presetColor)}
      >
        {isSelected && <CheckIcon $color={presetColor} />}
      </SelectableColorSwatch>
    );
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
        <ColorList>{mainColors.map(renderColorSwatch)}</ColorList>
        <ColorGrid>
          {toneArrays.map((tones) => (
            <ColorGridCol key={tones.join("-")}>
              {tones.map(renderColorSwatch)}
            </ColorGridCol>
          ))}
        </ColorGrid>
      </ColorsWrapper>
      <HorizontalDivider />
      <RecentLabel>{t("color_picker.recent")}</RecentLabel>
      <RecentColorsList>
        {recentColors.current.length > 0 ? (
          recentColors.current.map((recentColor) => (
            <ColorSwatch
              key={recentColor}
              $color={recentColor}
              onClick={(): void => {
                setSelectedColor(recentColor);
                handleColorSelect(recentColor);
              }}
            />
          ))
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

const StyledMenu = styled(Menu)({
  "& .MuiPaper-root": {
    borderRadius: 8,
    padding: "4px 0px",
    marginLeft: -4,
    maxWidth: 220,
  },
  "& .MuiList-root": {
    padding: 0,
  },
});

const MenuItemWrapper = styled(MenuItem)({
  display: "flex",
  flexDirection: "row",
  justifyContent: "flex-start",
  fontSize: 12,
  gap: 8,
  width: "calc(100% - 8px)",
  minWidth: 172,
  margin: "0px 4px 4px 4px",
  borderRadius: 4,
  padding: 8,
  height: 32,
});

const MenuItemText = styled("div")(({ theme }) => ({
  color: theme.palette.text.primary,
}));

const MenuItemSquare = styled("div")(({ theme }) => ({
  width: 16,
  height: 16,
  boxSizing: "border-box",
  marginTop: 0,
  border: `1px solid ${theme.palette.grey[300]}`,
  borderRadius: 4,
}));

const ColorsWrapper = styled("div")({
  display: "flex",
  flexDirection: "column",
  margin: 4,
});

const ColorList = styled("div")({
  display: "flex",
  flexWrap: "wrap",
  flexDirection: "row",
  margin: "8px 8px 0px 8px",
  justifyContent: "flex-start",
  gap: 4,
});

const ColorGrid = styled("div")({
  display: "flex",
  flexDirection: "row",
  justifyContent: "flex-start",
  margin: 8,
  gap: 4,
});

const ColorGridCol = styled("div")({
  display: "flex",
  flexDirection: "column",
  justifyContent: "flex-start",
  gap: 4,
});

const ColorSwatch = styled("button")<{ $color: string }>(
  ({ $color, theme }) => {
    const upperColor = $color.toUpperCase();

    return {
      width: 16,
      height: 16,
      padding: 0,
      border:
        upperColor === "#FFFFFF" || upperColor === "#FFF"
          ? `1px solid ${theme.palette.grey[300]}`
          : "none",
      backgroundColor: $color === "transparent" ? "transparent" : $color,
      boxSizing: "border-box",
      marginTop: 0,
      borderRadius: 4,

      "&:hover": {
        cursor: "pointer",
        outline: `1px solid ${theme.palette.grey[300]}`,
        outlineOffset: 1,
      },
    };
  },
);

const SelectableColorSwatch = styled(ColorSwatch)({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
});

// This function checks if a color is light or dark.
// This is needed to determine the text color for the check icon, as it's not visible on light colors.
const isLightColor = (hex: string): boolean => {
  const n = parseInt(hex.slice(1), 16);
  const r = (n >> 16) & 255;
  const g = (n >> 8) & 255;
  const b = n & 255;

  // We use luminance weighting to determine if the color is light or dark
  // (https://en.wikipedia.org/wiki/Relative_luminance). The threshold of 160 (out of max ~255)
  // means: if the calculated luminance is above 160, the color is considered "light" and a black
  // checkmark is used. Otherwise, a white checkmark ensures visibility on darker backgrounds.
  const luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
  return luminance > 160;
};

const CheckIcon = styled(Check)<{ $color: string }>(({ $color, theme }) => ({
  width: 10,
  height: 10,
  strokeWidth: 3,
  color: isLightColor($color)
    ? theme.palette.common.black
    : theme.palette.common.white,
}));

const HorizontalDivider = styled("div")(({ theme }) => ({
  height: 0,
  width: "100%",
  borderTop: `1px solid ${theme.palette.grey[200]}`,
}));

const RecentLabel = styled("div")(({ theme }) => ({
  fontFamily: "Inter",
  fontSize: 12,
  margin: "8px 12px 0px 12px",
  color: theme.palette.text.secondary,
}));

const RecentColorsList = styled("div")({
  display: "flex",
  flexWrap: "wrap",
  flexDirection: "row",
  padding: 8,
  margin: "0px 4px",
  justifyContent: "flex-start",
  gap: 4,
});

const StyledPlusButton = styled("button")(({ theme }) => ({
  display: "flex",
  justifyContent: "center",
  flexWrap: "wrap",
  alignItems: "center",
  border: "none",
  background: "none",
  fontSize: 12,
  height: 16,
  width: 16,
  margin: 0,
  padding: 0,
  borderRadius: 4,

  "& svg": {
    width: 16,
    height: 16,
  },

  "&:hover": {
    cursor: "pointer",
    outline: `1px solid ${theme.palette.grey[300]}`,
    outlineOffset: 1,
  },
}));

const EmptyContainer = styled("div")({
  display: "none",
});

export default ColorPicker;
