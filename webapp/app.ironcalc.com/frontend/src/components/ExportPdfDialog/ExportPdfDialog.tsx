import { Button, Checkbox, IconButton, type Model } from "@ironcalc/workbook";
import { ChevronDown, ChevronUp, Loader2, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { createPortal } from "react-dom";
import {
  collectFontFamiliesFromModel,
  fetchFontsForFamilies,
  uint8ArrayToBase64,
} from "../fontUtils";
import {
  type CustomFontPayload,
  fetchModelPdfBlob,
  type PdfExportPayloadOptions,
  triggerBlobDownload,
} from "../rpc";
import { PdfPagesPreview } from "./PdfPagesPreview";
import "./export-pdf-dialog.css";

function columnNumberToName(columnNumber: number): string {
  let columnName = "";
  let num = columnNumber;
  while (num > 0) {
    const modulo = (num - 1) % 26;
    columnName = String.fromCharCode(65 + modulo) + columnName;
    num = Math.floor((num - modulo) / 26);
  }
  return columnName;
}

interface ExportPdfDialogProperties {
  open: boolean;
  onClose: () => void;
  model: Model;
}

export function ExportPdfDialog({
  open,
  onClose,
  model,
}: ExportPdfDialogProperties) {
  // Selected range detection from model
  const selectionInfo = useMemo(() => {
    if (!open || !model) return null;
    try {
      if (typeof model.getSelectedView === "function") {
        const view = model.getSelectedView();
        if (view && Array.isArray(view.range) && view.range.length === 4) {
          const [r1, c1, r2, c2] = view.range;
          const rowStart = Math.min(r1, r2);
          const rowEnd = Math.max(r1, r2);
          const colStart = Math.min(c1, c2);
          const colEnd = Math.max(c1, c2);
          const startColStr = columnNumberToName(colStart);
          const endColStr = columnNumberToName(colEnd);
          const rangeStr =
            rowStart === rowEnd && colStart === colEnd
              ? `${startColStr}${rowStart}`
              : `${startColStr}${rowStart}:${endColStr}${rowEnd}`;
          return {
            sheetIndex: view.sheet,
            rangeStr,
          };
        }
      }
      if (typeof model.getSelectedCell === "function") {
        const cell = model.getSelectedCell();
        if (cell && cell.length >= 2) {
          const row = cell[0];
          const col = cell[1];
          const colStr = columnNumberToName(col);
          const currentSheet = model.getSelectedSheet ? model.getSelectedSheet() : 0;
          return {
            sheetIndex: currentSheet,
            rangeStr: `${colStr}${row}`,
          };
        }
      }
    } catch (e) {
      console.warn("Could not determine selected cells:", e);
    }
    return null;
  }, [open, model]);

  // Basic Page Setup
  const [scope, setScope] = useState<string>("CurrentSheet");
  const [paperSize, setPaperSize] = useState<string>("Letter");
  const [customWidth, setCustomWidth] = useState<number>(21.6);
  const [customHeight, setCustomHeight] = useState<number>(27.9);
  const [orientation, setOrientation] = useState<"Landscape" | "Portrait">("Landscape");
  const [scale, setScale] = useState<string>("FitToWidth");
  const [scalePercentage, setScalePercentage] = useState<number>(100);
  const [margins, setMargins] = useState<string>("Normal");
  const [customMarginTop, setCustomMarginTop] = useState<number>(1.9);
  const [customMarginRight, setCustomMarginRight] = useState<number>(1.8);
  const [customMarginBottom, setCustomMarginBottom] = useState<number>(1.9);
  const [customMarginLeft, setCustomMarginLeft] = useState<number>(1.8);

  // Accordion Sections
  const [formattingOpen, setFormattingOpen] = useState<boolean>(true);
  const [headersFootersOpen, setHeadersFootersOpen] = useState<boolean>(false);

  // Formatting Options
  const [showGridLines, setShowGridLines] = useState<boolean>(true);
  const [showNotes, setShowNotes] = useState<boolean>(false);
  const [pageOrder, setPageOrder] = useState<string>("DownThenOver");
  const [horizontalAlign, setHorizontalAlign] = useState<string>("Left");
  const [verticalAlign, setVerticalAlign] = useState<string>("Top");

  // Headers and Footers
  const [pageNumbers, setPageNumbers] = useState<boolean>(false);
  const [workbookTitle, setWorkbookTitle] = useState<boolean>(false);
  const [sheetName, setSheetName] = useState<boolean>(false);
  const [currentDate, setCurrentDate] = useState<boolean>(false);
  const [currentTime, setCurrentTime] = useState<boolean>(false);
  const [repeatFrozenRows, setRepeatFrozenRows] = useState<boolean>(false);
  const [repeatFrozenColumns, setRepeatFrozenColumns] = useState<boolean>(false);

  // State for fonts
  const [loadedCustomFonts, setLoadedCustomFonts] = useState<CustomFontPayload[]>([]);
  const [fontsLoaded, setFontsLoaded] = useState<boolean>(false);

  // State for live preview
  const [previewBlob, setPreviewBlob] = useState<Blob | null>(null);
  const [isLoadingPreview, setIsLoadingPreview] = useState<boolean>(true);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const [pageCount, setPageCount] = useState<number>(0);

  // State for exporting progress
  const [isExporting, setIsExporting] = useState<boolean>(false);
  const [statusText, setStatusText] = useState<string>("");

  // Initialize sheet-dependent defaults when opening
  useEffect(() => {
    if (open && model) {
      try {
        const currentSheet = model.getSelectedSheet ? model.getSelectedSheet() : 0;
        if (typeof model.getShowGridLines === "function") {
          setShowGridLines(model.getShowGridLines(currentSheet));
        }
      } catch (e) {
        console.warn("Could not read sheet properties:", e);
      }
    }
  }, [open, model]);

  // Load custom fonts once when dialog opens
  useEffect(() => {
    if (!open || !model) return;
    let cancelled = false;

    async function loadFonts() {
      try {
        const families = collectFontFamiliesFromModel(model);
        const { fontMap } = await fetchFontsForFamilies(families);
        if (cancelled) return;

        const fonts: CustomFontPayload[] = [];
        fontMap.forEach((bytes, name) => {
          fonts.push({
            name,
            data: uint8ArrayToBase64(bytes),
          });
        });
        setLoadedCustomFonts(fonts);
        setFontsLoaded(true);
      } catch (err) {
        console.warn("Could not fetch fonts for preview:", err);
        if (!cancelled) setFontsLoaded(true);
      }
    }

    loadFonts();
    return () => {
      cancelled = true;
    };
  }, [open, model]);


  // Escape key handler
  useEffect(() => {
    if (!open) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [open, onClose]);

  // Build payload options
  const exportOptions: PdfExportPayloadOptions = useMemo(() => {
    const currentSheet = model.getSelectedSheet ? model.getSelectedSheet() : 0;
    const isSelectedScope = scope === "SelectedCells";
    const targetSheet =
      isSelectedScope && selectionInfo ? selectionInfo.sheetIndex : currentSheet;
    const targetRange =
      isSelectedScope && selectionInfo ? selectionInfo.rangeStr : undefined;

    const opts: PdfExportPayloadOptions = {
      scope,
      sheet_index: targetSheet,
      selection_range: targetRange,
      paper_size: paperSize,
      orientation,
      scale,
      scale_percentage: scalePercentage,
      margins,
      show_grid_lines: showGridLines,
      page_order: pageOrder,
      horizontal_alignment: horizontalAlign,
      vertical_alignment: verticalAlign,
      page_numbers: pageNumbers,
      workbook_title: workbookTitle,
      sheet_name: sheetName,
      current_date: currentDate,
      current_time: currentTime,
      repeat_frozen_rows: repeatFrozenRows,
      repeat_frozen_columns: repeatFrozenColumns,
    };

    if (paperSize === "Custom") {
      opts.custom_width_pt = (customWidth / 2.54) * 72;
      opts.custom_height_pt = (customHeight / 2.54) * 72;
    }

    if (margins === "Custom") {
      opts.margin_top_pt = (customMarginTop / 2.54) * 72;
      opts.margin_right_pt = (customMarginRight / 2.54) * 72;
      opts.margin_bottom_pt = (customMarginBottom / 2.54) * 72;
      opts.margin_left_pt = (customMarginLeft / 2.54) * 72;
    }

    return opts;
  }, [
    model,
    scope,
    selectionInfo,
    paperSize,
    orientation,
    scale,
    scalePercentage,
    margins,
    showGridLines,
    pageOrder,
    horizontalAlign,
    verticalAlign,
    pageNumbers,
    workbookTitle,
    sheetName,
    currentDate,
    currentTime,
    repeatFrozenRows,
    repeatFrozenColumns,
    customWidth,
    customHeight,
    customMarginTop,
    customMarginRight,
    customMarginBottom,
    customMarginLeft,
  ]);

  // Debounced live preview generation
  useEffect(() => {
    if (!open || !fontsLoaded) return;

    let cancelled = false;
    setIsLoadingPreview(true);
    setPreviewError(null);

    const timer = setTimeout(async () => {
      try {
        const bytes = model.toBytes();
        const blob = await fetchModelPdfBlob(bytes, exportOptions, loadedCustomFonts);
        if (cancelled) return;

        setPreviewBlob(blob);
        setIsLoadingPreview(false);
      } catch (err) {
        if (cancelled) return;
        console.error("Preview generation error:", err);
        setPreviewError(err instanceof Error ? err.message : String(err));
        setIsLoadingPreview(false);
      }
    }, 250);

    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [open, fontsLoaded, exportOptions, loadedCustomFonts, model]);

  const handleRetryPreview = () => {
    setIsLoadingPreview(true);
    setPreviewError(null);
    const bytes = model.toBytes();
    fetchModelPdfBlob(bytes, exportOptions, loadedCustomFonts)
      .then((blob) => {
        setPreviewBlob(blob);
        setIsLoadingPreview(false);
      })
      .catch((err) => {
        setPreviewError(err instanceof Error ? err.message : String(err));
        setIsLoadingPreview(false);
      });
  };

  if (!open) {
    return null;
  }

  const handleExport = async () => {
    setIsExporting(true);
    setStatusText("Downloading PDF...");

    try {
      const fileName = model.getName();
      if (previewBlob && !isLoadingPreview) {
        // Fast path: use already generated and verified blob!
        triggerBlobDownload(previewBlob, fileName);
        onClose();
      } else {
        const bytes = model.toBytes();
        const blob = await fetchModelPdfBlob(bytes, exportOptions, loadedCustomFonts);
        triggerBlobDownload(blob, fileName);
        onClose();
      }
    } catch (error) {
      console.error("PDF Export error:", error);
      setStatusText(
        `Export failed: ${error instanceof Error ? error.message : String(error)}`,
      );
    } finally {
      setIsExporting(false);
    }
  };

  return createPortal(
    <div
      className="app-ic-epd-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="Export to PDF"
      onClick={onClose}
    >
      {/* Left side: Live PDF Preview Area */}
      <div
        className="app-ic-epd-preview-area"
        role="region"
        aria-label="PDF Preview"
      >
        <PdfPagesPreview
          blob={previewBlob}
          isLoading={isLoadingPreview}
          error={previewError}
          onRetry={handleRetryPreview}
          onPageCountChange={setPageCount}
          onBackgroundClick={onClose}
        />
      </div>

      {/* Right side: Export Settings Sheet */}
      <div
        className="app-ic-epd-panel"
        onClick={(e) => e.stopPropagation()}
        role="region"
        aria-label="Export settings panel"
      >
        <div className="app-ic-epd-header">
          <span className="app-ic-epd-header-title">Export settings</span>
          <IconButton
            variant="ghost"
            size="sm"
            icon={<X size={16} />}
            aria-label="Close dialog"
            onClick={onClose}
          />
        </div>

        <div className="app-ic-epd-body">
          {/* Scope */}
          <div className="app-ic-epd-field">
            <label className="app-ic-epd-label">Export</label>
            <select
              className="app-ic-epd-select"
              value={scope}
              onChange={(e) => setScope(e.target.value)}
            >
              <option value="CurrentSheet">Current sheet</option>
              <option value="Workbook">Workbook</option>
              <option value="SelectedCells">
                Selected cells{selectionInfo ? ` (${selectionInfo.rangeStr})` : ""}
              </option>
            </select>
          </div>

          {/* Paper Size */}
          <div className="app-ic-epd-field">
            <label className="app-ic-epd-label">Paper size</label>
            <select
              className="app-ic-epd-select"
              value={paperSize}
              onChange={(e) => setPaperSize(e.target.value)}
            >
              <option value="Letter">Letter (21.6 cm x 27.9 cm)</option>
              <option value="Tabloid">Tabloid (27.9 cm x 43.2 cm)</option>
              <option value="Legal">Legal (21.6 cm x 35.6 cm)</option>
              <option value="Statement">Statement (14.0 cm x 21.6 cm)</option>
              <option value="Executive">Executive (18.4 cm x 26.7 cm)</option>
              <option value="Folio">Folio (21.6 cm x 33.0 cm)</option>
              <option value="A3">A3 (29.7 cm x 42.0 cm)</option>
              <option value="A4">A4 (21.0 cm x 29.7 cm)</option>
              <option value="A5">A5 (14.8 cm x 21.0 cm)</option>
              <option value="B4">B4 (25.0 cm x 35.3 cm)</option>
              <option value="B5">B5 (17.6 cm x 25.0 cm)</option>
              <option value="Custom">Custom size</option>
            </select>
          </div>

          {paperSize === "Custom" && (
            <div className="app-ic-epd-row-2">
              <div className="app-ic-epd-field">
                <label className="app-ic-epd-label">Width (cm)</label>
                <input
                  type="number"
                  className="app-ic-epd-input"
                  value={customWidth}
                  onChange={(e) => setCustomWidth(Number(e.target.value))}
                  step="0.1"
                  min="5"
                />
              </div>
              <div className="app-ic-epd-field">
                <label className="app-ic-epd-label">Height (cm)</label>
                <input
                  type="number"
                  className="app-ic-epd-input"
                  value={customHeight}
                  onChange={(e) => setCustomHeight(Number(e.target.value))}
                  step="0.1"
                  min="5"
                />
              </div>
            </div>
          )}

          {/* Orientation */}
          <div className="app-ic-epd-field">
            <label className="app-ic-epd-label">Page orientation</label>
            <div className="app-ic-epd-radio-group">
              <label className="app-ic-epd-radio-label">
                <input
                  type="radio"
                  name="orientation"
                  value="Landscape"
                  checked={orientation === "Landscape"}
                  onChange={() => setOrientation("Landscape")}
                />
                Landscape
              </label>
              <label className="app-ic-epd-radio-label">
                <input
                  type="radio"
                  name="orientation"
                  value="Portrait"
                  checked={orientation === "Portrait"}
                  onChange={() => setOrientation("Portrait")}
                />
                Portrait
              </label>
            </div>
          </div>

          {/* Scale */}
          <div className="app-ic-epd-field">
            <label className="app-ic-epd-label">Scale</label>
            <div className="app-ic-epd-row-2">
              <select
                className="app-ic-epd-select"
                value={scale}
                onChange={(e) => setScale(e.target.value)}
              >
                <option value="FitToWidth">Fit to width</option>
                <option value="Normal">Normal (100%)</option>
                <option value="FitToHeight">Fit to height</option>
                <option value="FitToPage">Fit to page</option>
                <option value="Custom">Custom number</option>
              </select>
              {scale === "Custom" && (
                <input
                  type="number"
                  className="app-ic-epd-input"
                  placeholder="Percentage"
                  value={scalePercentage}
                  onChange={(e) => setScalePercentage(Number(e.target.value))}
                  min="10"
                  max="400"
                />
              )}
            </div>
          </div>

          {/* Margins */}
          <div className="app-ic-epd-field">
            <label className="app-ic-epd-label">Margins</label>
            <select
              className="app-ic-epd-select"
              value={margins}
              onChange={(e) => setMargins(e.target.value)}
            >
              <option value="Normal">Normal</option>
              <option value="Narrow">Narrow</option>
              <option value="Wide">Wide</option>
              <option value="Custom">Custom numbers</option>
            </select>
          </div>

          {margins === "Custom" && (
            <div className="app-ic-epd-row-4">
              <div className="app-ic-epd-field">
                <label className="app-ic-epd-label">Top</label>
                <input
                  type="number"
                  className="app-ic-epd-input"
                  value={customMarginTop}
                  onChange={(e) => setCustomMarginTop(Number(e.target.value))}
                  step="0.1"
                />
              </div>
              <div className="app-ic-epd-field">
                <label className="app-ic-epd-label">Right</label>
                <input
                  type="number"
                  className="app-ic-epd-input"
                  value={customMarginRight}
                  onChange={(e) => setCustomMarginRight(Number(e.target.value))}
                  step="0.1"
                />
              </div>
              <div className="app-ic-epd-field">
                <label className="app-ic-epd-label">Bottom</label>
                <input
                  type="number"
                  className="app-ic-epd-input"
                  value={customMarginBottom}
                  onChange={(e) => setCustomMarginBottom(Number(e.target.value))}
                  step="0.1"
                />
              </div>
              <div className="app-ic-epd-field">
                <label className="app-ic-epd-label">Left</label>
                <input
                  type="number"
                  className="app-ic-epd-input"
                  value={customMarginLeft}
                  onChange={(e) => setCustomMarginLeft(Number(e.target.value))}
                  step="0.1"
                />
              </div>
            </div>
          )}

          {/* Accordion: Formatting */}
          <div className="app-ic-epd-accordion">
            <div
              className="app-ic-epd-accordion-header"
              onClick={() => setFormattingOpen(!formattingOpen)}
            >
              <span>Formatting</span>
              {formattingOpen ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
            </div>

            {formattingOpen && (
              <div className="app-ic-epd-accordion-content">
                <Checkbox
                  checked={showGridLines}
                  onChange={setShowGridLines}
                  label="Show grid lines"
                />

                <Checkbox
                  checked={showNotes}
                  onChange={setShowNotes}
                  label="Show notes"
                />

                <div className="app-ic-epd-field">
                  <label className="app-ic-epd-label">Page order</label>
                  <select
                    className="app-ic-epd-select"
                    value={pageOrder}
                    onChange={(e) => setPageOrder(e.target.value)}
                  >
                    <option value="DownThenOver">Down, then over</option>
                    <option value="OverThenDown">Over, then down</option>
                  </select>
                </div>

                <div className="app-ic-epd-row-2">
                  <div className="app-ic-epd-field">
                    <label className="app-ic-epd-label">Horizontal</label>
                    <select
                      className="app-ic-epd-select"
                      value={horizontalAlign}
                      onChange={(e) => setHorizontalAlign(e.target.value)}
                    >
                      <option value="Left">Left</option>
                      <option value="Center">Center</option>
                      <option value="Right">Right</option>
                    </select>
                  </div>
                  <div className="app-ic-epd-field">
                    <label className="app-ic-epd-label">Vertical</label>
                    <select
                      className="app-ic-epd-select"
                      value={verticalAlign}
                      onChange={(e) => setVerticalAlign(e.target.value)}
                    >
                      <option value="Top">Top</option>
                      <option value="Middle">Middle</option>
                      <option value="Bottom">Bottom</option>
                    </select>
                  </div>
                </div>
              </div>
            )}
          </div>

          {/* Accordion: Headers and footers */}
          <div className="app-ic-epd-accordion">
            <div
              className="app-ic-epd-accordion-header"
              onClick={() => setHeadersFootersOpen(!headersFootersOpen)}
            >
              <span>Headers and footers</span>
              {headersFootersOpen ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
            </div>

            {headersFootersOpen && (
              <div className="app-ic-epd-accordion-content">
                <Checkbox
                  checked={pageNumbers}
                  onChange={setPageNumbers}
                  label="Page numbers"
                />

                <Checkbox
                  checked={workbookTitle}
                  onChange={setWorkbookTitle}
                  label="Workbook title"
                />

                <Checkbox
                  checked={sheetName}
                  onChange={setSheetName}
                  label="Sheet name"
                />

                <Checkbox
                  checked={currentDate}
                  onChange={setCurrentDate}
                  label="Current date"
                />

                <Checkbox
                  checked={currentTime}
                  onChange={setCurrentTime}
                  label="Current time"
                />

                <div className="app-ic-epd-subheading">Row & column headers</div>
                <div className="app-ic-epd-hint">
                  Go to View &gt; Freeze to select which rows/columns to repeat on all pages
                </div>

                <Checkbox
                  checked={repeatFrozenRows}
                  onChange={setRepeatFrozenRows}
                  label="Repeat frozen rows"
                />

                <Checkbox
                  checked={repeatFrozenColumns}
                  onChange={setRepeatFrozenColumns}
                  label="Repeat frozen columns"
                />
              </div>
            )}
          </div>

          {statusText && (
            <div className="app-ic-epd-status">
              {isExporting && <Loader2 size={16} className="app-ic-epd-spin" />}
              <span>{statusText}</span>
            </div>
          )}
        </div>

        <div className="app-ic-epd-footer">
          <span className="app-ic-epd-page-count">
            {pageCount > 0 ? `${pageCount} ${pageCount === 1 ? "page" : "pages"}` : ""}
          </span>
          <div className="app-ic-epd-footer-actions">
            <Button
              type="button"
              variant="secondary"
              size="md"
              onClick={onClose}
              disabled={isExporting}
            >
              Cancel
            </Button>
            <Button
              type="button"
              variant="primary"
              size="md"
              onClick={handleExport}
              disabled={isExporting}
              startIcon={
                isExporting ? <Loader2 size={15} className="app-ic-epd-spin" /> : undefined
              }
            >
              {isExporting ? "Exporting..." : "Export"}
            </Button>
          </div>
        </div>
      </div>
    </div>,
    document.body,
  );
}
