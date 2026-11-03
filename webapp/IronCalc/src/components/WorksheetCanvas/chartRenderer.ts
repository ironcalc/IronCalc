import type { AnchorPoint, Model } from "@ironcalc/wasm";
import { HANDLE_CURSORS, HANDLES } from "./chartInteraction";

// English Metric Units per CSS pixel at 96 dpi
const EMU_PER_PIXEL = 9525;

/** What the chart layer needs from the worksheet canvas. */
export interface ChartHost {
  model: Model;
  selectedSheet(): number;
  /** Top-left corner of a cell relative to the canvas */
  getCoordinatesByCell(row: number, column: number): [number, number];
  /** The scrolling pane: x, y, width, height relative to the canvas */
  getScrollPaneRect(): [number, number, number, number];
  /** The chart that shows the selection frame, if any */
  selectedChart: { sheet: number; id: number } | null;
  /** Accessible name of the chart menu button on the frame */
  menuButtonLabel: string;
}

const MENU_ICON =
  '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/></svg>';

/** The selection frame: a border with resize handles and a menu button. */
function createFrame(): HTMLDivElement {
  const frame = document.createElement("div");
  frame.className = "ic-worksheet-chart-frame";
  for (const handle of HANDLES) {
    const element = document.createElement("div");
    element.className = "ic-worksheet-chart-handle";
    element.dataset.handle = handle;
    element.style.cursor = HANDLE_CURSORS[handle];
    frame.appendChild(element);
  }
  const button = document.createElement("button");
  button.type = "button";
  button.className = "ic-worksheet-chart-menu-button";
  button.innerHTML = MENU_ICON;
  frame.appendChild(button);
  return frame;
}

// The SVG each element currently shows, to skip rewriting it when nothing
// changed. Keyed by the element itself: the layer is shared by every canvas
// instance that is created for the same worksheet element.
const shownSvg: WeakMap<HTMLDivElement, string> = new WeakMap();

/**
 * Shows the charts of the selected sheet as SVG elements floating over the
 * scrolling pane of the canvas.
 *
 * The engine lays every chart out and returns it as SVG; this only places
 * the elements and swaps their content when it changes. The layer clips to
 * the scrolling pane, so a chart under a frozen pane is cut off there.
 *
 * The elements are reconciled against the layer's own children, keyed by
 * sheet and chart id, so nothing is left behind when the sheet changes or
 * when a new canvas takes over the layer.
 */
export class ChartLayer {
  private layer: HTMLDivElement;

  constructor(layer: HTMLDivElement) {
    this.layer = layer;
  }

  render(host: ChartHost): void {
    const sheet = host.selectedSheet();
    const [paneX, paneY, paneWidth, paneHeight] = host.getScrollPaneRect();
    const { style } = this.layer;
    style.left = `${paneX}px`;
    style.top = `${paneY}px`;
    style.width = `${paneWidth}px`;
    style.height = `${paneHeight}px`;

    const corner = (point: AnchorPoint): [number, number] => {
      const [x, y] = host.getCoordinatesByCell(point.row, point.column);
      return [
        x + point.column_offset / EMU_PER_PIXEL,
        y + point.row_offset / EMU_PER_PIXEL,
      ];
    };

    // What is in the layer now (the frame has no chart key)
    const existing: Map<string, HTMLDivElement> = new Map();
    for (const child of Array.from(this.layer.children)) {
      const div = child as HTMLDivElement;
      if (div.dataset.chartKey) {
        existing.set(div.dataset.chartKey, div);
      }
    }
    const selected = host.selectedChart;
    const selectedKey =
      selected && selected.sheet === sheet
        ? `${selected.sheet}:${selected.id}`
        : null;
    let selectedRect: [number, number, number, number] | null = null;

    const onScreen = new Set<string>();
    for (const id of host.model.getChartIds(sheet)) {
      const anchor = host.model.getDrawingAnchor(sheet, id);
      const [x1, y1] = corner(anchor.from);
      const [x2, y2] = corner(anchor.to);
      const width = Math.max(1, Math.round(x2 - x1));
      const height = Math.max(1, Math.round(y2 - y1));
      const left = Math.round(x1 - paneX);
      const top = Math.round(y1 - paneY);
      if (
        left + width <= 0 ||
        top + height <= 0 ||
        left >= paneWidth ||
        top >= paneHeight
      ) {
        continue;
      }
      const key = `${sheet}:${id}`;
      let div = existing.get(key);
      if (!div) {
        div = document.createElement("div");
        div.className = "ic-worksheet-chart";
        div.dataset.chartKey = key;
        div.dataset.chartId = `${id}`;
        this.layer.appendChild(div);
      }
      div.style.left = `${left}px`;
      div.style.top = `${top}px`;
      div.style.width = `${width}px`;
      div.style.height = `${height}px`;
      const isSelected = key === selectedKey;
      div.classList.toggle("ic-worksheet-chart--selected", isSelected);
      if (isSelected) {
        selectedRect = [left, top, width, height];
      }
      const svg = host.model.getChartSvg(sheet, id, width, height);
      if (svg !== shownSvg.get(div)) {
        div.innerHTML = svg;
        shownSvg.set(div, svg);
      }
      onScreen.add(key);
    }

    for (const [key, div] of existing) {
      if (!onScreen.has(key)) {
        div.remove();
      }
    }

    // The selection frame, always last so it paints over the charts
    let frame = this.layer.querySelector<HTMLDivElement>(
      ".ic-worksheet-chart-frame",
    );
    if (!frame) {
      frame = createFrame();
    }
    this.layer.appendChild(frame);
    if (selected && selectedRect) {
      const [left, top, width, height] = selectedRect;
      frame.style.display = "";
      frame.style.left = `${left}px`;
      frame.style.top = `${top}px`;
      frame.style.width = `${width}px`;
      frame.style.height = `${height}px`;
      frame.dataset.chartId = `${selected.id}`;
      const button = frame.querySelector<HTMLButtonElement>(
        ".ic-worksheet-chart-menu-button",
      );
      if (button) {
        button.dataset.chartId = `${selected.id}`;
        button.setAttribute("aria-label", host.menuButtonLabel);
        button.title = host.menuButtonLabel;
      }
    } else {
      frame.style.display = "none";
    }
  }
}
