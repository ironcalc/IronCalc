export interface SheetOptions {
  name: string;
  // CSS color of the tab as shown on screen
  color: string;
  // False when the sheet has no tab color (or a white one)
  hasColor: boolean;
  sheetId: number;
  state: string;
}
