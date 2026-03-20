import "@mui/material/styles";

type SheetPalette = {
  headerCornerBackground: string;
  headerTextColor: string;
  headerBackground: string;
  headerGlobalSelectorColor: string;
  headerSelectedBackground: string;
  headerFullSelectedBackground: string;
  headerSelectedColor: string;
  headerBorderColor: string;
  gridColor: string;
  gridSeparatorColor: string;
  defaultTextColor: string;
  outlineColor: string;
  outlineEditingColor: string;
  outlineBackgroundColor: string;
  defaultCellFontFamily: string;
  headerFontFamily: string;
  headerFontSize: 12;
};

declare module "@mui/material/styles" {
  interface Palette {
    sheet: SheetPalette;
  }

  interface PaletteOptions {
    sheet?: Partial<SheetPalette>;
  }
}
