import type {
  Anchor,
  AnchorBehavior,
  AnchorPoint,
  Chart,
  ChartSetup,
  Color,
  DataLabelPosition,
  DataLabels,
  DrawingStyle,
  Font,
  Gridlines,
  IronCalcTheme,
  Line,
  LineStyle,
  Model,
  Position,
  Stacking,
  TextDirection,
  TickMark,
  TitleAlignment,
} from "@ironcalc/wasm";
import {
  AlignCenter,
  AlignLeft,
  AlignRight,
  ArrowLeftRight,
  Bold,
  ChartColumnBig,
  ChevronRight,
  Italic,
  Minus,
  Plus,
  SquareMousePointer,
  X,
} from "lucide-react";
import {
  Fragment,
  type KeyboardEvent,
  type ReactNode,
  type PointerEvent as ReactPointerEvent,
  useEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { IconButton } from "../../Button/IconButton";
import { Checkbox } from "../../Checkbox/Checkbox";
import ColorPicker from "../../ColorPicker/ColorPicker";
import { resolveColorToHex } from "../../ColorPicker/util";
import { Input } from "../../Input/Input";
import { Select } from "../../Select/Select";
import { Switch } from "../../Switch/Switch";
import { ToggleButton } from "../../ToggleButton/ToggleButton";
import { Tooltip } from "../../Tooltip/Tooltip";
import {
  automaticLine,
  automaticSeriesColor,
  defaultBars,
  defaultChartTitle,
  font as defaultFont,
  emuToPixels,
  formatCell,
  fromPickerColor,
  manualToRange,
  noLine,
  parseCell,
  pixelsToEmu,
  rangeToManual,
  seriesNameFormula,
  seriesNameText,
  toPickerColor,
} from "./chartUtil";
import "./chart-editor.css";

const FONT_SIZES = [8, 9, 10, 11, 12, 14, 16, 18, 20, 24, 28, 32];
// Excel's angles for the category labels, positive counter-clockwise
const LABEL_ANGLES = [90, 60, 45, 30, 0, -30, -45, -60, -90];
const AUTOMATIC_ANGLE = "auto";
const LINE_STYLES: LineStyle[] = [
  "Automatic",
  "None",
  "Thin",
  "Thick",
  "Dashed",
];
// The docked positions of a legend; "None" is the legend's switch
const LEGEND_POSITIONS = ["Top", "Right", "Bottom", "Left"] as const;
const LABEL_POSITIONS: DataLabelPosition[] = [
  "Center",
  "InsideEnd",
  "InsideBase",
  "OutsideEnd",
];
const TICK_MARKS: TickMark[] = ["None", "Inside", "Outside", "Cross"];
const TEXT_DIRECTIONS: TextDirection[] = [
  "Horizontal",
  "Rotate90",
  "Rotate270",
];

/** The labels a series gets when they are switched on. */
function defaultDataLabels(stacking: Stacking): DataLabels {
  return {
    position: stacking === "None" ? "OutsideEnd" : "Center",
    direction: "Horizontal",
    number_format: null,
    font: defaultFont(9),
    fill: null,
    border: automaticLine,
  };
}

interface ChartEditorProps {
  model: Model;
  sheet: number;
  id: number;
  currentTheme: IronCalcTheme;
  onClose: () => void;
  /** The chart changed: the sheet needs a repaint */
  onUpdate: () => void;
  /** The range selected on the sheet, as `Sheet1!$A$1:$B$2` */
  getSelectedArea: () => string;
}

type Tab = "setup" | "properties";

/** The collapsible sections of the properties tab. */
type SectionName =
  | "title"
  | "legend"
  | "data_labels"
  | "bars"
  | "gridlines"
  | "value_axis"
  | "category_axis"
  | "chart_area"
  | "position"
  | "hidden_and_empty";

/** A text field that applies its value on blur or Enter, and reverts on Escape. */
function CommitInput({
  value,
  onCommit,
  disabled = false,
  placeholder,
  ariaLabel,
  endAdornment,
}: {
  value: string;
  onCommit: (value: string) => void;
  disabled?: boolean;
  placeholder?: string;
  ariaLabel?: string;
  endAdornment?: ReactNode;
}) {
  const [draft, setDraft] = useState(value);
  // A new value from the model replaces the draft
  useEffect(() => {
    setDraft(value);
  }, [value]);
  const commit = () => {
    if (draft !== value) {
      onCommit(draft);
    }
  };
  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter") {
      commit();
      event.currentTarget.blur();
    } else if (event.key === "Escape") {
      setDraft(value);
      event.currentTarget.blur();
    }
    event.stopPropagation();
  };
  return (
    <Input
      size="sm"
      value={draft}
      disabled={disabled}
      placeholder={placeholder}
      aria-label={ariaLabel}
      onChange={(event) => setDraft(event.target.value)}
      onBlur={commit}
      onKeyDown={onKeyDown}
      endAdornment={endAdornment}
    />
  );
}

/** The button that puts the range selected on the sheet into a field. */
function UseSelectionButton({
  onPick,
  disabled = false,
}: {
  onPick: () => void;
  disabled?: boolean;
}) {
  const { t } = useTranslation();
  if (disabled) {
    return null;
  }
  return (
    <Tooltip title={t("chart_editor.use_selection")}>
      <IconButton
        size="sm"
        variant="secondary"
        icon={<SquareMousePointer />}
        aria-label={t("chart_editor.use_selection")}
        onClick={onPick}
        className="ic-chart-editor-range-button"
      />
    </Tooltip>
  );
}

/** A colored square that opens the color picker. */
function ColorSwatch({
  color,
  automatic,
  title,
  theme,
  onChange,
}: {
  color: Color | null | undefined;
  /** The color shown, and restored, when there is none */
  automatic: string;
  title: string;
  theme: IronCalcTheme;
  onChange: (color: Color | null) => void;
}) {
  const [open, setOpen] = useState(false);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const shown =
    color === null || color === undefined
      ? automatic
      : resolveColorToHex(color, theme);
  return (
    <>
      <Tooltip title={title}>
        <button
          type="button"
          ref={buttonRef}
          className="ic-chart-editor-swatch"
          style={{ backgroundColor: shown }}
          aria-label={title}
          onClick={() => setOpen(true)}
        />
      </Tooltip>
      <ColorPicker
        color={toPickerColor(color)}
        defaultColor=""
        defaultSwatchColor={automatic}
        title={title}
        onChange={(picked) => {
          onChange(fromPickerColor(picked));
          setOpen(false);
        }}
        onClose={() => setOpen(false)}
        anchorEl={buttonRef}
        open={open}
        theme={theme}
      />
    </>
  );
}

const ANCHOR_BEHAVIORS: AnchorBehavior[] = [
  "MoveAndResize",
  "MoveOnly",
  "Fixed",
];

/** A corner of the frame: the cell and the offset into it, in pixels. */
function AnchorRow({
  label,
  point,
  onChange,
}: {
  label: string;
  point: AnchorPoint;
  onChange: (point: AnchorPoint) => void;
}) {
  const { t } = useTranslation();
  const offset = (text: string, fallback: number) => {
    const value = Number.parseInt(text, 10);
    return Number.isNaN(value) || value < 0 ? fallback : pixelsToEmu(value);
  };
  return (
    <Row label={label}>
      <div className="ic-chart-editor-anchor-row">
        <CommitInput
          value={formatCell(point.row, point.column)}
          ariaLabel={`${label} ${t("chart_editor.anchor_cell")}`}
          onCommit={(text) => {
            const cell = parseCell(text);
            if (cell) {
              onChange({ ...point, row: cell.row, column: cell.column });
            }
          }}
        />
        <CommitInput
          value={`${emuToPixels(point.column_offset)}`}
          placeholder="x"
          ariaLabel={`${label} ${t("chart_editor.anchor_offset_x")}`}
          onCommit={(text) =>
            onChange({
              ...point,
              column_offset: offset(text, point.column_offset),
            })
          }
        />
        <CommitInput
          value={`${emuToPixels(point.row_offset)}`}
          placeholder="y"
          ariaLabel={`${label} ${t("chart_editor.anchor_offset_y")}`}
          onCommit={(text) =>
            onChange({ ...point, row_offset: offset(text, point.row_offset) })
          }
        />
      </div>
    </Row>
  );
}

/** A percentage typed by the user, clamped to its range; `fallback` when it is not a number. */
function percent(
  text: string,
  fallback: number,
  min: number,
  max: number,
): number {
  const value = Number.parseInt(text.replace("%", ""), 10);
  return Number.isNaN(value) ? fallback : Math.min(Math.max(value, min), max);
}

/** Excel's angles, plus the chart's own when it is none of them. */
function labelAngles(rotation: number | null): number[] {
  if (rotation === null || LABEL_ANGLES.includes(rotation)) {
    return LABEL_ANGLES;
  }
  return [...LABEL_ANGLES, rotation].sort((a, b) => b - a);
}

function Row({ label, children }: { label: ReactNode; children: ReactNode }) {
  return (
    <div className="ic-chart-editor-row">
      <div className="ic-chart-editor-row-label">{label}</div>
      <div className="ic-chart-editor-row-control">{children}</div>
    </div>
  );
}

/**
 * A titled group of rows. With `expanded` it is collapsible: a chevron in
 * front of the title shows the state, and `onToggle` flips it. Without
 * `onToggle` the section is switched off and stays closed.
 */
function Section({
  title,
  action,
  expanded,
  onToggle,
  children,
}: {
  title: ReactNode;
  action?: ReactNode;
  expanded?: boolean;
  onToggle?: () => void;
  children: ReactNode;
}) {
  const collapsible = expanded !== undefined;
  const open = !collapsible || expanded;
  return (
    <div className="ic-chart-editor-section">
      <div className="ic-chart-editor-section-header">
        {collapsible ? (
          <button
            type="button"
            className="ic-chart-editor-section-toggle"
            aria-expanded={open}
            disabled={!onToggle}
            onClick={onToggle}
          >
            <ChevronRight
              size={16}
              className="ic-chart-editor-section-chevron"
            />
            <span className="ic-chart-editor-section-title">{title}</span>
          </button>
        ) : (
          <div className="ic-chart-editor-section-title">{title}</div>
        )}
        {action}
      </div>
      {open && children}
    </div>
  );
}

/** Size, bold, italic and color of a font, in one line. */
function FontRow({
  font,
  theme,
  automaticColor,
  onChange,
}: {
  font: Font;
  theme: IronCalcTheme;
  automaticColor: string;
  onChange: (font: Font) => void;
}) {
  const { t } = useTranslation();
  const styles: ("bold" | "italic")[] = [];
  if (font.bold) {
    styles.push("bold");
  }
  if (font.italic) {
    styles.push("italic");
  }
  const sizes = FONT_SIZES.includes(font.size)
    ? FONT_SIZES
    : [...FONT_SIZES, font.size].sort((a, b) => a - b);
  return (
    <div className="ic-chart-editor-font-row">
      <Select
        size="sm"
        value={`${font.size}`}
        options={sizes.map((size) => ({ value: `${size}`, label: `${size}` }))}
        onChange={(value) => onChange({ ...font, size: Number(value) })}
        className="ic-chart-editor-font-size"
      />
      <ToggleButton
        multiple
        value={styles}
        options={[
          {
            value: "bold",
            icon: <Bold />,
            "aria-label": t("chart_editor.bold"),
          },
          {
            value: "italic",
            icon: <Italic />,
            "aria-label": t("chart_editor.italic"),
          },
        ]}
        onChange={(values) =>
          onChange({
            ...font,
            bold: values.includes("bold"),
            italic: values.includes("italic"),
          })
        }
      />
      <ColorSwatch
        color={font.color}
        automatic={automaticColor}
        title={t("chart_editor.color")}
        theme={theme}
        onChange={(color) => onChange({ ...font, color })}
      />
    </div>
  );
}

// The legend proxy on the position pad, as fractions of the pad
const PAD_LEGEND_WIDTH = 0.22;
const PAD_LEGEND_HEIGHT = 0.16;

/**
 * A miniature of the chart on which the legend is dragged into place. The
 * position is the top-left corner of the legend as fractions of the chart.
 * The drag previews live and commits once, on release; a click on the pad
 * centers the legend there.
 */
function LegendPositionPad({
  x,
  y,
  hint,
  onChange,
}: {
  x: number;
  y: number;
  hint: string;
  onChange: (x: number, y: number) => void;
}) {
  const padRef = useRef<HTMLDivElement>(null);
  const [preview, setPreview] = useState<{ x: number; y: number } | null>(null);
  // The pointer's offset from the legend corner while dragging it
  const grab = useRef<{ dx: number; dy: number } | null>(null);
  const shown = preview ?? { x, y };

  const clamp = (fx: number, fy: number) => ({
    x: Math.min(Math.max(fx, 0), 1 - PAD_LEGEND_WIDTH),
    y: Math.min(Math.max(fy, 0), 1 - PAD_LEGEND_HEIGHT),
  });
  const fractionAt = (event: ReactPointerEvent) => {
    const pad = padRef.current;
    if (!pad) {
      return null;
    }
    const rect = pad.getBoundingClientRect();
    return {
      x: (event.clientX - rect.left) / rect.width,
      y: (event.clientY - rect.top) / rect.height,
    };
  };

  const onPadPointerDown = (event: ReactPointerEvent<HTMLDivElement>) => {
    const point = fractionAt(event);
    if (!point) {
      return;
    }
    const onLegend = (event.target as HTMLElement).classList.contains(
      "ic-chart-editor-legend-proxy",
    );
    if (onLegend) {
      grab.current = { dx: point.x - shown.x, dy: point.y - shown.y };
    } else {
      // Center the legend on the click
      grab.current = { dx: PAD_LEGEND_WIDTH / 2, dy: PAD_LEGEND_HEIGHT / 2 };
      setPreview(clamp(point.x - grab.current.dx, point.y - grab.current.dy));
    }
    event.currentTarget.setPointerCapture(event.pointerId);
    event.preventDefault();
  };
  const onPadPointerMove = (event: ReactPointerEvent<HTMLDivElement>) => {
    const point = fractionAt(event);
    if (!grab.current || !point) {
      return;
    }
    setPreview(clamp(point.x - grab.current.dx, point.y - grab.current.dy));
  };
  const onPadPointerUp = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (!grab.current) {
      return;
    }
    grab.current = null;
    event.currentTarget.releasePointerCapture(event.pointerId);
    const final = preview ?? { x, y };
    setPreview(null);
    onChange(
      Math.round(final.x * 1000) / 1000,
      Math.round(final.y * 1000) / 1000,
    );
  };

  return (
    <div className="ic-chart-editor-legend-pad-wrapper">
      <div
        ref={padRef}
        className="ic-chart-editor-legend-pad"
        title={hint}
        onPointerDown={onPadPointerDown}
        onPointerMove={onPadPointerMove}
        onPointerUp={onPadPointerUp}
        onPointerCancel={onPadPointerUp}
      >
        <div className="ic-chart-editor-legend-pad-plot" />
        <div
          className="ic-chart-editor-legend-proxy"
          style={{
            left: `${shown.x * 100}%`,
            top: `${shown.y * 100}%`,
            width: `${PAD_LEGEND_WIDTH * 100}%`,
            height: `${PAD_LEGEND_HEIGHT * 100}%`,
          }}
        />
      </div>
      <div className="ic-chart-editor-legend-pad-values">
        {`${Math.round(shown.x * 100)}%, ${Math.round(shown.y * 100)}%`}
      </div>
    </div>
  );
}

/** Style and color of a line, in one line. */
function LineRow({
  line,
  theme,
  automaticColor,
  onChange,
}: {
  line: Line;
  theme: IronCalcTheme;
  automaticColor: string;
  onChange: (line: Line) => void;
}) {
  const { t } = useTranslation();
  return (
    <div className="ic-chart-editor-font-row">
      <Select
        size="sm"
        value={line.style}
        options={LINE_STYLES.map((style) => ({
          value: style,
          label: t(`chart_editor.line_${style.toLowerCase()}`),
        }))}
        onChange={(value) => onChange({ ...line, style: value as LineStyle })}
        className="ic-chart-editor-line-style"
      />
      <ColorSwatch
        color={line.color}
        automatic={automaticColor}
        title={t("chart_editor.color")}
        theme={theme}
        onChange={(color) => onChange({ ...line, color })}
      />
    </div>
  );
}

const ChartEditor = ({
  model,
  sheet,
  id,
  currentTheme,
  onClose,
  onUpdate,
  getSelectedArea,
}: ChartEditorProps) => {
  const { t } = useTranslation();
  const [tab, setTab] = useState<Tab>("setup");
  // The series whose data labels the editor shows: an index, or every series
  const [labelTarget, setLabelTarget] = useState<string>("all");
  // The sections of the properties tab that are open; all start closed
  const [expanded, setExpanded] = useState<
    Partial<Record<SectionName, boolean>>
  >({});
  const expand = (name: SectionName, open: boolean) =>
    setExpanded((previous) => ({ ...previous, [name]: open }));
  // The props that make a section collapsible; a switched-off one stays closed
  const section = (name: SectionName, enabled = true) => ({
    expanded: enabled && (expanded[name] ?? false),
    onToggle: enabled ? () => expand(name, !expanded[name]) : undefined,
  });

  // The chart may be gone (undo, another delete) while the drawer is open
  let chart: Chart | null = null;
  let setup: ChartSetup | null = null;
  let drawingStyle: DrawingStyle | null = null;
  let anchor: Anchor | null = null;
  try {
    chart = model.getChart(sheet, id);
    setup = model.getChartSetup(sheet, id);
    drawingStyle = model.getDrawingStyle(sheet, id);
    anchor = model.getDrawingAnchor(sheet, id);
  } catch {
    chart = null;
  }

  const header = (
    <div className="ic-chart-editor-header">
      <div className="ic-chart-editor-header-title">
        {t("chart_editor.title")}
      </div>
      <Tooltip title={t("right_drawer.close")}>
        <IconButton
          icon={<X />}
          onClick={onClose}
          aria-label={t("right_drawer.close")}
        />
      </Tooltip>
    </div>
  );

  if (!chart || !setup || !drawingStyle || !anchor) {
    return (
      <div className="ic-chart-editor">
        {header}
        <div className="ic-chart-editor-empty">
          {t("chart_editor.no_chart")}
        </div>
      </div>
    );
  }

  // Every change goes to the model and repaints the sheet
  const update = (change: (next: Chart) => void) => {
    if (!chart) {
      return;
    }
    const next = structuredClone(chart);
    change(next);
    model.updateChart(sheet, id, next);
    onUpdate();
  };

  // The references of the data source, with a leading "=" dropped
  const setRange = (text: string) =>
    update((next) => {
      if (next.data.range.type === "Range") {
        next.data.range.range = text.replace(/^=/, "");
      }
    });
  const setSeriesValues = (index: number, text: string) =>
    update((next) => {
      if (next.data.range.type === "Manual") {
        next.data.range.series[index].values = text.replace(/^=/, "");
      }
    });
  const setCategories = (text: string) =>
    update((next) => {
      if (next.data.range.type === "Manual") {
        next.data.range.categories = text.replace(/^=/, "");
      }
    });

  // Moves the frame, keeping the bottom-right corner after the top-left one
  const moveTo = (next: Anchor) => {
    const rowsOrdered =
      next.to.row > next.from.row ||
      (next.to.row === next.from.row &&
        next.to.row_offset > next.from.row_offset);
    const columnsOrdered =
      next.to.column > next.from.column ||
      (next.to.column === next.from.column &&
        next.to.column_offset > next.from.column_offset);
    if (!rowsOrdered || !columnsOrdered) {
      return;
    }
    model.moveDrawing(sheet, id, next);
    onUpdate();
  };

  // Excel's gray for text without a color, and the theme's dark for lines
  const automaticTextColor = resolveColorToHex([1, 0.35], currentTheme);
  const automaticLineColor = resolveColorToHex([1, 0.85], currentTheme);
  // Excel's automatic minor gridline is lighter than the major one
  const automaticMinorLineColor = resolveColorToHex([1, 0.95], currentTheme);

  const { data } = chart;
  const isRange = data.range.type === "Range";
  const orientation =
    data.range.type === "Range" ? data.range.orientation : null;

  const seriesRows = setup.series.map((series, index) => {
    const name = isRange ? series.label : seriesNameText(series.name);
    const color = chart.colors[index] ?? null;
    return (
      <div
        className={[
          "ic-chart-editor-series-row",
          isRange ? "ic-chart-editor-series-row--range" : "",
        ].join(" ")}
        // biome-ignore lint/suspicious/noArrayIndexKey: series have no identity besides their position
        key={index}
      >
        <CommitInput
          value={name}
          disabled={isRange}
          placeholder={t("chart_editor.series_name")}
          ariaLabel={t("chart_editor.series_name")}
          onCommit={(text) =>
            update((next) => {
              if (next.data.range.type === "Manual") {
                next.data.range.series[index].name = seriesNameFormula(text);
              }
            })
          }
        />
        <CommitInput
          value={series.values}
          disabled={isRange}
          placeholder="Sheet1!$B$2:$B$10"
          ariaLabel={t("chart_editor.series_values")}
          onCommit={(text) => setSeriesValues(index, text)}
          endAdornment={
            <UseSelectionButton
              disabled={isRange}
              onPick={() => setSeriesValues(index, getSelectedArea())}
            />
          }
        />
        <ColorSwatch
          color={color}
          automatic={automaticSeriesColor(index, currentTheme)}
          title={t("chart_editor.series_color")}
          theme={currentTheme}
          onChange={(picked) =>
            update((next) => {
              while (next.colors.length <= index) {
                next.colors.push(null);
              }
              next.colors[index] = picked;
            })
          }
        />
        {!isRange && (
          <Tooltip title={t("chart_editor.remove_series")}>
            <IconButton
              icon={<Minus />}
              aria-label={t("chart_editor.remove_series")}
              onClick={() =>
                update((next) => {
                  if (next.data.range.type === "Manual") {
                    next.data.range.series.splice(index, 1);
                    if (index < next.colors.length) {
                      next.colors.splice(index, 1);
                    }
                    if (index < next.data_labels.length) {
                      next.data_labels.splice(index, 1);
                    }
                  }
                })
              }
            />
          </Tooltip>
        )}
      </div>
    );
  });

  const setupTab = (
    <>
      <div className="ic-chart-editor-section">
        <Row label={t("chart_editor.chart_type")}>
          <Select
            size="sm"
            value="column"
            disabled
            options={[
              {
                value: "column",
                label: (
                  <span className="ic-chart-editor-type-option">
                    <ChartColumnBig size={16} />
                    {t("chart_editor.type_column")}
                  </span>
                ),
              },
            ]}
            onChange={() => {}}
          />
        </Row>
        <Row label={t("chart_editor.variant")}>
          <Select
            size="sm"
            value={chart.stacking}
            options={[
              { value: "None", label: t("chart_editor.variant_clustered") },
              { value: "Standard", label: t("chart_editor.variant_stacked") },
              { value: "Full", label: t("chart_editor.variant_percent") },
            ]}
            onChange={(value) =>
              update((next) => {
                const wasClustered = next.stacking === "None";
                next.stacking = value as Stacking;
                // Clustered and stacked columns have different spacing
                if (wasClustered !== (next.stacking === "None")) {
                  next.bars = defaultBars(next.stacking);
                }
                // Outside end only exists for clustered columns
                if (next.stacking !== "None") {
                  for (const labels of next.data_labels) {
                    if (labels?.position === "OutsideEnd") {
                      labels.position = "Center";
                    }
                  }
                }
              })
            }
          />
        </Row>
        <Row label={t("chart_editor.data_source")}>
          <ToggleButton
            fullWidth
            value={isRange ? "range" : "manual"}
            options={[
              { value: "range", label: t("chart_editor.source_range") },
              { value: "manual", label: t("chart_editor.source_manual") },
            ]}
            onChange={(value) =>
              update((next) => {
                if (value === "manual" && next.data.range.type === "Range") {
                  next.data.range = rangeToManual(
                    setup.categories,
                    setup.series,
                  );
                } else if (
                  value === "range" &&
                  next.data.range.type === "Manual"
                ) {
                  const { categories, series } = next.data.range;
                  const range = manualToRange(categories, series);
                  next.data.range = {
                    type: "Range",
                    range: range ? range.range : categories,
                    orientation: range ? range.orientation : "Columns",
                  };
                }
              })
            }
          />
        </Row>
        {data.range.type === "Range" && (
          <Row label="">
            <div className="ic-chart-editor-range-row">
              <CommitInput
                value={data.range.range}
                placeholder="Sheet1!$A$1:$D$10"
                ariaLabel={t("chart_editor.source_range")}
                onCommit={(text) => setRange(text)}
                endAdornment={
                  <UseSelectionButton
                    onPick={() => setRange(getSelectedArea())}
                  />
                }
              />
              <Tooltip title={t("chart_editor.switch_rows_columns")}>
                <IconButton
                  icon={<ArrowLeftRight />}
                  aria-label={t("chart_editor.switch_rows_columns")}
                  pressed={orientation === "Rows"}
                  onClick={() =>
                    update((next) => {
                      if (next.data.range.type === "Range") {
                        next.data.range.orientation =
                          next.data.range.orientation === "Columns"
                            ? "Rows"
                            : "Columns";
                      }
                    })
                  }
                />
              </Tooltip>
            </div>
          </Row>
        )}
      </div>
      <div className="ic-drawer-divider" />
      <Section
        title={t("chart_editor.series")}
        action={
          !isRange && (
            <Tooltip title={t("chart_editor.add_series")}>
              <IconButton
                icon={<Plus />}
                aria-label={t("chart_editor.add_series")}
                onClick={() =>
                  update((next) => {
                    if (next.data.range.type === "Manual") {
                      next.data.range.series.push({ name: "", values: "" });
                    }
                  })
                }
              />
            </Tooltip>
          )
        }
      >
        {seriesRows.length > 0 && (
          <div
            className={[
              "ic-chart-editor-series-header",
              isRange ? "ic-chart-editor-series-header--range" : "",
            ].join(" ")}
          >
            <div>{t("chart_editor.series_name")}</div>
            <div>{t("chart_editor.series_values")}</div>
          </div>
        )}
        {seriesRows}
        <div className="ic-chart-editor-section-title ic-chart-editor-category-title">
          {t("chart_editor.category")}
        </div>
        <CommitInput
          value={setup.categories}
          disabled={isRange}
          placeholder="Sheet1!$A$2:$A$10"
          ariaLabel={t("chart_editor.category")}
          onCommit={(text) => setCategories(text)}
          endAdornment={
            <UseSelectionButton
              disabled={isRange}
              onPick={() => setCategories(getSelectedArea())}
            />
          }
        />
      </Section>
    </>
  );

  const legendPosition: string =
    typeof chart.legend.position === "string"
      ? chart.legend.position
      : "Custom";
  const legendOptions: { value: string; label: string }[] =
    LEGEND_POSITIONS.map((position) => ({
      value: position,
      label: t(`chart_editor.position_${position.toLowerCase()}`),
    }));
  legendOptions.push({
    value: "Custom",
    label: t("chart_editor.position_custom"),
  });
  // Where a legend goes when it starts floating: about where it was docked
  const floatingStart = (position: Position): [number, number] => {
    switch (position) {
      case "Top":
        return [0.4, 0.05];
      case "Right":
        return [0.8, 0.4];
      case "Left":
        return [0.02, 0.4];
      default:
        return [0.4, 0.88];
    }
  };
  const valueFormat = chart.value_axis.labels.number_format;

  // Data labels: which series the section edits, and what it shows. "All"
  // shows the first labelled series and writes to every series.
  const seriesCount = setup.series.length;
  const labelTargets: number[] =
    labelTarget === "all"
      ? Array.from({ length: seriesCount }, (_, index) => index)
      : [Number(labelTarget)].filter((index) => index < seriesCount);
  const currentLabels: DataLabels | null =
    labelTarget === "all"
      ? (chart.data_labels.find((labels) => labels !== null) ?? null)
      : (chart.data_labels[Number(labelTarget)] ?? null);
  const labelsEnabled =
    labelTargets.length > 0 &&
    labelTargets.every((index) => chart.data_labels[index]);
  // Applies `change` to the labels of every target series, creating them
  // with the defaults where a series has none
  const setLabels = (change: (labels: DataLabels) => void) =>
    update((next) => {
      for (const index of labelTargets) {
        while (next.data_labels.length <= index) {
          next.data_labels.push(null);
        }
        const labels =
          next.data_labels[index] ?? defaultDataLabels(next.stacking);
        change(labels);
        next.data_labels[index] = labels;
      }
    });

  const tickOptions = TICK_MARKS.map((tick) => ({
    value: tick,
    label: t(`chart_editor.tick_${tick.toLowerCase()}`),
  }));
  // Whether any gridlines show; the switch of the section clears them all
  const gridlinesShown = [chart.value_axis, chart.category_axis].some(
    (axis) =>
      axis.gridlines.major.style !== "None" ||
      axis.gridlines.minor.style !== "None",
  );
  // The gridlines of one axis, under the axis's name
  const axisGridlines = (axis: "value_axis" | "category_axis") => {
    const gridlines = chart[axis].gridlines;
    const set = (change: (gridlines: Gridlines) => void) =>
      update((next) => change(next[axis].gridlines));
    return (
      <Fragment key={axis}>
        <div className="ic-chart-editor-section-title ic-chart-editor-subsection-title">
          {t(`chart_editor.${axis}`)}
        </div>
        <Row label={t("chart_editor.major_gridlines")}>
          <LineRow
            line={gridlines.major}
            theme={currentTheme}
            automaticColor={automaticLineColor}
            onChange={(line) =>
              set((gridlines) => {
                gridlines.major = line;
              })
            }
          />
        </Row>
        <Row label={t("chart_editor.minor_gridlines")}>
          <LineRow
            line={gridlines.minor}
            theme={currentTheme}
            automaticColor={automaticMinorLineColor}
            onChange={(line) =>
              set((gridlines) => {
                gridlines.minor = line;
              })
            }
          />
        </Row>
      </Fragment>
    );
  };

  const propertiesTab = (
    <>
      <Section
        title={t("chart_editor.chart_title")}
        {...section("title", chart.title !== null)}
        action={
          <Switch
            checked={chart.title !== null}
            aria-label={t("chart_editor.chart_title")}
            onChange={(checked) => {
              update((next) => {
                next.title = checked ? defaultChartTitle() : null;
              });
              expand("title", checked);
            }}
          />
        }
      >
        <Row label={t("chart_editor.title_text")}>
          <CommitInput
            value={chart.title?.text ?? ""}
            ariaLabel={t("chart_editor.title_text")}
            onCommit={(text) =>
              update((next) => {
                if (text === "") {
                  next.title = null;
                } else {
                  next.title = {
                    text,
                    font: next.title?.font ?? defaultFont(14),
                    alignment: next.title?.alignment ?? "Middle",
                    fill: next.title?.fill ?? null,
                    border: next.title?.border ?? automaticLine,
                    subtitle: next.title?.subtitle ?? null,
                  };
                }
              })
            }
          />
        </Row>
        {chart.title && (
          <>
            <Row label={t("chart_editor.alignment")}>
              <ToggleButton
                value={chart.title.alignment}
                options={[
                  {
                    value: "Left",
                    icon: <AlignLeft />,
                    "aria-label": t("toolbar.align_left"),
                  },
                  {
                    value: "Middle",
                    icon: <AlignCenter />,
                    "aria-label": t("toolbar.align_center"),
                  },
                  {
                    value: "Right",
                    icon: <AlignRight />,
                    "aria-label": t("toolbar.align_right"),
                  },
                ]}
                onChange={(value) =>
                  update((next) => {
                    if (next.title) {
                      next.title.alignment = value as TitleAlignment;
                    }
                  })
                }
              />
            </Row>
            <Row label={t("chart_editor.font")}>
              <FontRow
                font={chart.title.font}
                theme={currentTheme}
                automaticColor={automaticTextColor}
                onChange={(font) =>
                  update((next) => {
                    if (next.title) {
                      next.title.font = font;
                    }
                  })
                }
              />
            </Row>
            <Row label={t("chart_editor.subtitle")}>
              <CommitInput
                value={chart.title.subtitle?.text ?? ""}
                ariaLabel={t("chart_editor.subtitle")}
                onCommit={(text) =>
                  update((next) => {
                    if (next.title) {
                      next.title.subtitle =
                        text === ""
                          ? null
                          : {
                              text,
                              font:
                                next.title.subtitle?.font ?? defaultFont(10),
                            };
                    }
                  })
                }
              />
            </Row>
            {chart.title.subtitle && (
              <Row label={t("chart_editor.subtitle_font")}>
                <FontRow
                  font={chart.title.subtitle.font}
                  theme={currentTheme}
                  automaticColor={automaticTextColor}
                  onChange={(font) =>
                    update((next) => {
                      if (next.title?.subtitle) {
                        next.title.subtitle.font = font;
                      }
                    })
                  }
                />
              </Row>
            )}
            <Row label={t("chart_editor.fill")}>
              <div className="ic-chart-editor-font-row ic-chart-editor-swatch-row">
                <ColorSwatch
                  color={chart.title.fill}
                  automatic="transparent"
                  title={t("chart_editor.fill")}
                  theme={currentTheme}
                  onChange={(color) =>
                    update((next) => {
                      if (next.title) {
                        next.title.fill = color;
                      }
                    })
                  }
                />
              </div>
            </Row>
            <Row label={t("chart_editor.border")}>
              <LineRow
                line={chart.title.border}
                theme={currentTheme}
                automaticColor={automaticLineColor}
                onChange={(line) =>
                  update((next) => {
                    if (next.title) {
                      next.title.border = line;
                    }
                  })
                }
              />
            </Row>
          </>
        )}
      </Section>
      <div className="ic-drawer-divider" />
      <Section
        title={t("chart_editor.legend")}
        {...section("legend", chart.legend.position !== "None")}
        action={
          <Switch
            checked={chart.legend.position !== "None"}
            aria-label={t("chart_editor.legend")}
            onChange={(checked) => {
              update((next) => {
                next.legend.position = checked ? "Bottom" : "None";
                next.legend.vertical = false;
              });
              expand("legend", checked);
            }}
          />
        }
      >
        <Row label={t("chart_editor.legend_position")}>
          <Select
            size="sm"
            value={legendPosition}
            options={legendOptions}
            onChange={(value) =>
              update((next) => {
                if (value === "Custom") {
                  next.legend.position = {
                    Float: floatingStart(next.legend.position),
                  };
                } else {
                  const position = value as Exclude<
                    Position,
                    { Float: [number, number] }
                  >;
                  next.legend.position = position;
                  next.legend.vertical =
                    position === "Right" || position === "Left";
                }
              })
            }
          />
        </Row>
        {typeof chart.legend.position !== "string" && (
          <Row label="">
            <LegendPositionPad
              x={chart.legend.position.Float[0]}
              y={chart.legend.position.Float[1]}
              hint={t("chart_editor.legend_pad_hint")}
              onChange={(x, y) =>
                update((next) => {
                  next.legend.position = { Float: [x, y] };
                })
              }
            />
          </Row>
        )}
        <Checkbox
          checked={chart.legend.overlay}
          label={t("chart_editor.legend_overlay")}
          onChange={(checked) =>
            update((next) => {
              next.legend.overlay = checked;
            })
          }
        />
        <Row label={t("chart_editor.font")}>
          <FontRow
            font={chart.legend.font}
            theme={currentTheme}
            automaticColor={automaticTextColor}
            onChange={(font) =>
              update((next) => {
                next.legend.font = font;
              })
            }
          />
        </Row>
        <Row label={t("chart_editor.fill")}>
          <div className="ic-chart-editor-font-row ic-chart-editor-swatch-row">
            <ColorSwatch
              color={chart.legend.fill}
              automatic="transparent"
              title={t("chart_editor.fill")}
              theme={currentTheme}
              onChange={(color) =>
                update((next) => {
                  next.legend.fill = color;
                })
              }
            />
          </div>
        </Row>
        <Row label={t("chart_editor.border")}>
          <LineRow
            line={chart.legend.border}
            theme={currentTheme}
            automaticColor={automaticLineColor}
            onChange={(line) =>
              update((next) => {
                next.legend.border = line;
              })
            }
          />
        </Row>
      </Section>
      <div className="ic-drawer-divider" />
      <Section
        title={t("chart_editor.data_labels")}
        {...section("data_labels", labelsEnabled)}
        action={
          <Switch
            checked={labelsEnabled}
            disabled={seriesCount === 0}
            aria-label={t("chart_editor.data_labels")}
            onChange={(checked) => {
              update((next) => {
                for (const index of labelTargets) {
                  while (next.data_labels.length <= index) {
                    next.data_labels.push(null);
                  }
                  if (checked) {
                    next.data_labels[index] ??= defaultDataLabels(
                      next.stacking,
                    );
                  } else {
                    next.data_labels[index] = null;
                  }
                }
              });
              expand("data_labels", checked);
            }}
          />
        }
      >
        <Row label={t("chart_editor.labels_series")}>
          <Select
            size="sm"
            value={labelTarget}
            options={[
              { value: "all", label: t("chart_editor.all_series") },
              ...setup.series.map((series, index) => ({
                value: `${index}`,
                label:
                  series.label !== ""
                    ? series.label
                    : `${t("chart_editor.series")} ${index + 1}`,
              })),
            ]}
            onChange={setLabelTarget}
          />
        </Row>
        {currentLabels && (
          <>
            <Row label={t("chart_editor.label_position")}>
              <Select
                size="sm"
                value={currentLabels.position}
                options={LABEL_POSITIONS.filter(
                  (position) =>
                    position !== "OutsideEnd" || chart.stacking === "None",
                ).map((position) => ({
                  value: position,
                  label: t(`chart_editor.label_${position.toLowerCase()}`),
                }))}
                onChange={(value) =>
                  setLabels((labels) => {
                    labels.position = value as DataLabelPosition;
                  })
                }
              />
            </Row>
            <Row label={t("chart_editor.direction")}>
              <Select
                size="sm"
                value={currentLabels.direction}
                options={TEXT_DIRECTIONS.map((direction) => ({
                  value: direction,
                  label: t(`chart_editor.direction_${direction.toLowerCase()}`),
                }))}
                onChange={(value) =>
                  setLabels((labels) => {
                    labels.direction = value as TextDirection;
                  })
                }
              />
            </Row>
            <Row label={t("chart_editor.number_format")}>
              <Select
                size="sm"
                value={currentLabels.number_format === null ? "auto" : "custom"}
                options={[
                  { value: "auto", label: t("chart_editor.format_auto") },
                  { value: "custom", label: t("chart_editor.format_custom") },
                ]}
                onChange={(value) =>
                  setLabels((labels) => {
                    labels.number_format =
                      value === "auto"
                        ? null
                        : (currentLabels.number_format ?? "#,##0");
                  })
                }
              />
            </Row>
            {currentLabels.number_format !== null && (
              <Row label="">
                <CommitInput
                  value={currentLabels.number_format}
                  ariaLabel={t("chart_editor.number_format")}
                  onCommit={(text) =>
                    setLabels((labels) => {
                      labels.number_format = text;
                    })
                  }
                />
              </Row>
            )}
            <Row label={t("chart_editor.font")}>
              <FontRow
                font={currentLabels.font}
                theme={currentTheme}
                automaticColor={automaticTextColor}
                onChange={(font) =>
                  setLabels((labels) => {
                    labels.font = font;
                  })
                }
              />
            </Row>
            <Row label={t("chart_editor.fill")}>
              <div className="ic-chart-editor-font-row ic-chart-editor-swatch-row">
                <ColorSwatch
                  color={currentLabels.fill}
                  automatic="transparent"
                  title={t("chart_editor.fill")}
                  theme={currentTheme}
                  onChange={(color) =>
                    setLabels((labels) => {
                      labels.fill = color;
                    })
                  }
                />
              </div>
            </Row>
            <Row label={t("chart_editor.border")}>
              <LineRow
                line={currentLabels.border}
                theme={currentTheme}
                automaticColor={automaticLineColor}
                onChange={(line) =>
                  setLabels((labels) => {
                    labels.border = line;
                  })
                }
              />
            </Row>
          </>
        )}
      </Section>
      <div className="ic-drawer-divider" />
      <Section title={t("chart_editor.bars")} {...section("bars")}>
        <Row label={t("chart_editor.gap_width")}>
          <CommitInput
            value={`${chart.bars.gap}`}
            ariaLabel={t("chart_editor.gap_width")}
            endAdornment={<span className="ic-chart-editor-unit">%</span>}
            onCommit={(text) =>
              update((next) => {
                next.bars.gap = percent(text, next.bars.gap, 0, 500);
              })
            }
          />
        </Row>
        {/* Stacked columns always overlap fully */}
        <Row label={t("chart_editor.overlap")}>
          <CommitInput
            value={chart.stacking === "None" ? `${chart.bars.overlap}` : "100"}
            disabled={chart.stacking !== "None"}
            ariaLabel={t("chart_editor.overlap")}
            endAdornment={<span className="ic-chart-editor-unit">%</span>}
            onCommit={(text) =>
              update((next) => {
                next.bars.overlap = percent(text, next.bars.overlap, -100, 100);
              })
            }
          />
        </Row>
      </Section>
      <div className="ic-drawer-divider" />
      <Section title={t("chart_editor.value_axis")} {...section("value_axis")}>
        <Row label={t("chart_editor.axis_title")}>
          <CommitInput
            value={chart.value_axis.title?.text ?? ""}
            ariaLabel={t("chart_editor.axis_title")}
            onCommit={(text) =>
              update((next) => {
                next.value_axis.title =
                  text === ""
                    ? null
                    : {
                        text,
                        font: next.value_axis.title?.font ?? defaultFont(10),
                        alignment: "Middle",
                        fill: next.value_axis.title?.fill ?? null,
                        border: next.value_axis.title?.border ?? automaticLine,
                        subtitle: next.value_axis.title?.subtitle ?? null,
                      };
              })
            }
          />
        </Row>
        {chart.value_axis.title && (
          <>
            <Row label={t("chart_editor.title_font")}>
              <FontRow
                font={chart.value_axis.title.font}
                theme={currentTheme}
                automaticColor={automaticTextColor}
                onChange={(font) =>
                  update((next) => {
                    if (next.value_axis.title) {
                      next.value_axis.title.font = font;
                    }
                  })
                }
              />
            </Row>
            <Row label={t("chart_editor.title_fill")}>
              <div className="ic-chart-editor-font-row ic-chart-editor-swatch-row">
                <ColorSwatch
                  color={chart.value_axis.title.fill}
                  automatic="transparent"
                  title={t("chart_editor.title_fill")}
                  theme={currentTheme}
                  onChange={(color) =>
                    update((next) => {
                      if (next.value_axis.title) {
                        next.value_axis.title.fill = color;
                      }
                    })
                  }
                />
              </div>
            </Row>
            <Row label={t("chart_editor.title_border")}>
              <LineRow
                line={chart.value_axis.title.border}
                theme={currentTheme}
                automaticColor={automaticLineColor}
                onChange={(line) =>
                  update((next) => {
                    if (next.value_axis.title) {
                      next.value_axis.title.border = line;
                    }
                  })
                }
              />
            </Row>
          </>
        )}
        <Row label={t("chart_editor.number_format")}>
          <Select
            size="sm"
            value={valueFormat === null ? "auto" : "custom"}
            options={[
              { value: "auto", label: t("chart_editor.format_auto") },
              { value: "custom", label: t("chart_editor.format_custom") },
            ]}
            onChange={(value) =>
              update((next) => {
                next.value_axis.labels.number_format =
                  value === "auto" ? null : (valueFormat ?? "#,##0");
              })
            }
          />
        </Row>
        {valueFormat !== null && (
          <Row label="">
            <CommitInput
              value={valueFormat}
              ariaLabel={t("chart_editor.number_format")}
              onCommit={(text) =>
                update((next) => {
                  next.value_axis.labels.number_format = text;
                })
              }
            />
          </Row>
        )}
        <Row label={t("chart_editor.labels")}>
          <FontRow
            font={chart.value_axis.labels.font}
            theme={currentTheme}
            automaticColor={automaticTextColor}
            onChange={(font) =>
              update((next) => {
                next.value_axis.labels.font = font;
              })
            }
          />
        </Row>
        <Row label={t("chart_editor.axis_line")}>
          <LineRow
            line={chart.value_axis.line}
            theme={currentTheme}
            automaticColor={automaticLineColor}
            onChange={(line) =>
              update((next) => {
                next.value_axis.line = line;
              })
            }
          />
        </Row>
        <Row label={t("chart_editor.major_ticks")}>
          <Select
            size="sm"
            value={chart.value_axis.gridlines.major_ticks}
            options={tickOptions}
            onChange={(value) =>
              update((next) => {
                next.value_axis.gridlines.major_ticks = value as TickMark;
              })
            }
          />
        </Row>
        <Row label={t("chart_editor.minor_ticks")}>
          <Select
            size="sm"
            value={chart.value_axis.gridlines.minor_ticks}
            options={tickOptions}
            onChange={(value) =>
              update((next) => {
                next.value_axis.gridlines.minor_ticks = value as TickMark;
              })
            }
          />
        </Row>
      </Section>
      <div className="ic-drawer-divider" />
      <Section
        title={t("chart_editor.category_axis")}
        {...section("category_axis")}
      >
        <Row label={t("chart_editor.axis_title")}>
          <CommitInput
            value={chart.category_axis.title?.text ?? ""}
            ariaLabel={t("chart_editor.axis_title")}
            onCommit={(text) =>
              update((next) => {
                next.category_axis.title =
                  text === ""
                    ? null
                    : {
                        text,
                        font: next.category_axis.title?.font ?? defaultFont(10),
                        alignment: "Middle",
                        fill: next.category_axis.title?.fill ?? null,
                        border:
                          next.category_axis.title?.border ?? automaticLine,
                        subtitle: next.category_axis.title?.subtitle ?? null,
                      };
              })
            }
          />
        </Row>
        {chart.category_axis.title && (
          <>
            <Row label={t("chart_editor.title_font")}>
              <FontRow
                font={chart.category_axis.title.font}
                theme={currentTheme}
                automaticColor={automaticTextColor}
                onChange={(font) =>
                  update((next) => {
                    if (next.category_axis.title) {
                      next.category_axis.title.font = font;
                    }
                  })
                }
              />
            </Row>
            <Row label={t("chart_editor.title_fill")}>
              <div className="ic-chart-editor-font-row ic-chart-editor-swatch-row">
                <ColorSwatch
                  color={chart.category_axis.title.fill}
                  automatic="transparent"
                  title={t("chart_editor.title_fill")}
                  theme={currentTheme}
                  onChange={(color) =>
                    update((next) => {
                      if (next.category_axis.title) {
                        next.category_axis.title.fill = color;
                      }
                    })
                  }
                />
              </div>
            </Row>
            <Row label={t("chart_editor.title_border")}>
              <LineRow
                line={chart.category_axis.title.border}
                theme={currentTheme}
                automaticColor={automaticLineColor}
                onChange={(line) =>
                  update((next) => {
                    if (next.category_axis.title) {
                      next.category_axis.title.border = line;
                    }
                  })
                }
              />
            </Row>
          </>
        )}
        <Row label={t("chart_editor.labels")}>
          <FontRow
            font={chart.category_axis.labels}
            theme={currentTheme}
            automaticColor={automaticTextColor}
            onChange={(font) =>
              update((next) => {
                next.category_axis.labels = font;
              })
            }
          />
        </Row>
        <Row label={t("chart_editor.angle")}>
          <Select
            size="sm"
            value={
              chart.category_axis.rotation === null
                ? AUTOMATIC_ANGLE
                : `${chart.category_axis.rotation}`
            }
            options={[
              {
                value: AUTOMATIC_ANGLE,
                label: t("chart_editor.angle_automatic"),
              },
              ...labelAngles(chart.category_axis.rotation).map((angle) => ({
                value: `${angle}`,
                label: `${angle}°`,
              })),
            ]}
            onChange={(value) =>
              update((next) => {
                next.category_axis.rotation =
                  value === AUTOMATIC_ANGLE ? null : Number(value);
              })
            }
          />
        </Row>
        <Row label={t("chart_editor.axis_line")}>
          <LineRow
            line={chart.category_axis.line}
            theme={currentTheme}
            automaticColor={automaticLineColor}
            onChange={(line) =>
              update((next) => {
                next.category_axis.line = line;
              })
            }
          />
        </Row>
        <Row label={t("chart_editor.major_ticks")}>
          <Select
            size="sm"
            value={chart.category_axis.gridlines.major_ticks}
            options={tickOptions}
            onChange={(value) =>
              update((next) => {
                next.category_axis.gridlines.major_ticks = value as TickMark;
              })
            }
          />
        </Row>
        <Row label={t("chart_editor.minor_ticks")}>
          <Select
            size="sm"
            value={chart.category_axis.gridlines.minor_ticks}
            options={tickOptions}
            onChange={(value) =>
              update((next) => {
                next.category_axis.gridlines.minor_ticks = value as TickMark;
              })
            }
          />
        </Row>
      </Section>
      <div className="ic-drawer-divider" />
      <Section
        title={t("chart_editor.gridlines")}
        {...section("gridlines", gridlinesShown)}
        action={
          <Switch
            checked={gridlinesShown}
            aria-label={t("chart_editor.gridlines")}
            onChange={(checked) => {
              update((next) => {
                // On: Excel's default, major gridlines on the value axis
                next.value_axis.gridlines.major = checked
                  ? automaticLine
                  : noLine;
                next.value_axis.gridlines.minor = noLine;
                next.category_axis.gridlines.major = noLine;
                next.category_axis.gridlines.minor = noLine;
              });
              expand("gridlines", checked);
            }}
          />
        }
      >
        {axisGridlines("value_axis")}
        {axisGridlines("category_axis")}
      </Section>
      <div className="ic-drawer-divider" />
      <Section title={t("chart_editor.chart_area")} {...section("chart_area")}>
        <Row label={t("chart_editor.fill")}>
          <div className="ic-chart-editor-font-row">
            <Select
              size="sm"
              value={
                typeof drawingStyle.fill === "string"
                  ? drawingStyle.fill
                  : "Solid"
              }
              options={[
                { value: "Automatic", label: t("chart_editor.fill_automatic") },
                { value: "None", label: t("chart_editor.fill_none") },
                { value: "Solid", label: t("chart_editor.fill_solid") },
              ]}
              onChange={(value) => {
                const fill =
                  value === "Solid"
                    ? { Solid: currentTheme.lt1 }
                    : (value as "Automatic" | "None");
                model.setDrawingStyle(sheet, id, fill, drawingStyle.border);
                onUpdate();
              }}
              className="ic-chart-editor-line-style"
            />
            {typeof drawingStyle.fill !== "string" && (
              <ColorSwatch
                color={drawingStyle.fill.Solid}
                automatic={currentTheme.lt1}
                title={t("chart_editor.color")}
                theme={currentTheme}
                onChange={(color) => {
                  model.setDrawingStyle(
                    sheet,
                    id,
                    { Solid: color ?? currentTheme.lt1 },
                    drawingStyle.border,
                  );
                  onUpdate();
                }}
              />
            )}
          </div>
        </Row>
        <Row label={t("chart_editor.border")}>
          <LineRow
            line={drawingStyle.border}
            theme={currentTheme}
            automaticColor={automaticLineColor}
            onChange={(line) => {
              model.setDrawingStyle(sheet, id, drawingStyle.fill, line);
              onUpdate();
            }}
          />
        </Row>
      </Section>
      <div className="ic-drawer-divider" />
      <Section title={t("chart_editor.position")} {...section("position")}>
        <AnchorRow
          label={t("chart_editor.anchor_from")}
          point={anchor.from}
          onChange={(point) => moveTo({ ...anchor, from: point })}
        />
        <AnchorRow
          label={t("chart_editor.anchor_to")}
          point={anchor.to}
          onChange={(point) => moveTo({ ...anchor, to: point })}
        />
        <Row label={t("chart_editor.anchor_behavior")}>
          <Select
            size="sm"
            value={anchor.behavior}
            options={ANCHOR_BEHAVIORS.map((behavior) => ({
              value: behavior,
              label: t(`chart_editor.behavior_${behavior.toLowerCase()}`),
            }))}
            onChange={(value) =>
              moveTo({ ...anchor, behavior: value as AnchorBehavior })
            }
          />
        </Row>
      </Section>
      <div className="ic-drawer-divider" />
      <Section
        title={t("chart_editor.hidden_and_empty")}
        {...section("hidden_and_empty")}
      >
        <Row label={t("chart_editor.show_empty_as")}>
          <ToggleButton
            fullWidth
            value={data.empty_cells}
            options={[
              { value: "Gaps", label: t("chart_editor.empty_gaps") },
              { value: "Zero", label: t("chart_editor.empty_zero") },
            ]}
            onChange={(value) =>
              update((next) => {
                next.data.empty_cells = value;
              })
            }
          />
        </Row>
        <Checkbox
          checked={data.na_as_empty}
          label={t("chart_editor.na_as_empty")}
          onChange={(checked) =>
            update((next) => {
              next.data.na_as_empty = checked;
            })
          }
        />
        <Checkbox
          checked={data.show_hidden_data}
          label={t("chart_editor.show_hidden")}
          onChange={(checked) =>
            update((next) => {
              next.data.show_hidden_data = checked;
            })
          }
        />
      </Section>
    </>
  );

  return (
    <div className="ic-chart-editor">
      {header}
      <div className="ic-chart-editor-scroll">
        <div className="ic-chart-editor-preview">
          <div className="ic-chart-editor-preview-icon">
            <ChartColumnBig size={28} />
          </div>
          <div className="ic-chart-editor-preview-title">
            {t("chart_editor.column_chart")}
          </div>
        </div>
        <div className="ic-chart-editor-tabs" role="tablist">
          {(["setup", "properties"] as Tab[]).map((name) => (
            <button
              key={name}
              type="button"
              role="tab"
              aria-selected={tab === name}
              className={[
                "ic-chart-editor-tab",
                tab === name ? "ic-chart-editor-tab--active" : "",
              ].join(" ")}
              onClick={() => setTab(name)}
            >
              {t(`chart_editor.tab_${name}`)}
            </button>
          ))}
        </div>
        <div className="ic-chart-editor-content">
          {tab === "setup" ? setupTab : propertiesTab}
        </div>
      </div>
    </div>
  );
};

export default ChartEditor;
