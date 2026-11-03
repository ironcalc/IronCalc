// The chart layer: the charts of a sheet are SVG elements floating over the
// scrolling pane. The engine draws them; the layer only places them, so the
// tests check the geometry it assigns and that the content gets there.

import type { Chart } from "@ironcalc/wasm";
import { expect, test } from "vitest";
import {
  headerColumnWidth,
  headerRowHeight,
} from "../../src/components/WorksheetCanvas/constants";
import { WorkbookState } from "../../src/components/workbookState";
import { newModel, renderWorksheet } from "./harness";

const COLUMN_WIDTH = 90;
const ROW_HEIGHT = 25;

const px = (value: string): number => {
  expect(value).toMatch(/^-?[\d.]+px$/);
  return Number(value.slice(0, -2));
};

// The chart elements of the layer: its children except the selection frame
const chartElements = (layer: HTMLDivElement): HTMLDivElement[] =>
  (Array.from(layer.children) as HTMLDivElement[]).filter(
    (child) => child.dataset.chartId !== undefined,
  );

const rect = (element: HTMLElement) => ({
  left: px(element.style.left),
  top: px(element.style.top),
  width: px(element.style.width),
  height: px(element.style.height),
});

const font = {
  size: 9,
  family: null,
  bold: false,
  italic: false,
  color: null,
};
const line = { style: "Automatic", color: null } as const;
const noLine = { style: "None", color: null } as const;

function columnChart(title: string): Chart {
  return {
    type: "ColumnChart",
    title: {
      text: title,
      font,
      alignment: "Middle",
      fill: null,
      border: line,
      subtitle: null,
    },
    legend: {
      position: "Bottom",
      vertical: false,
      padding: null,
      fill: null,
      border: line,
      font,
      overlay: false,
    },
    stacking: "None",
    category_axis: {
      title: null,
      labels: font,
      rotation: null,
      line,
      gridlines: {
        major: noLine,
        minor: noLine,
        major_ticks: "None",
        minor_ticks: "None",
      },
    },
    value_axis: {
      title: null,
      labels: { number_format: null, font },
      line,
      gridlines: {
        major: line,
        minor: noLine,
        major_ticks: "None",
        minor_ticks: "None",
      },
    },
    data: {
      range: {
        type: "Range",
        range: "Sheet1!$A$1:$B$3",
        orientation: "Columns",
      },
      empty_cells: "Gaps",
      na_as_empty: false,
      show_hidden_data: false,
    },
    colors: [],
    data_labels: [],
    bars: { gap: 219, overlap: -27 },
  };
}

function anchor(row: number, column: number, rows: number, columns: number) {
  const point = (r: number, c: number) => ({
    row: r,
    column: c,
    row_offset: 0,
    column_offset: 0,
  });
  return {
    from: point(row, column),
    to: point(row + rows, column + columns),
    behavior: "MoveAndResize" as const,
  };
}

test("a chart floats over its anchor cells in the scrolling pane", async () => {
  const model = await newModel();
  model.setUserInput(0, 1, 1, "City");
  model.setUserInput(0, 1, 2, "Visitors");
  model.setUserInput(0, 2, 1, "London");
  model.setUserInput(0, 2, 2, "123");
  model.setUserInput(0, 3, 1, "Madrid");
  model.setUserInput(0, 3, 2, "456");
  // Rows 2 to 5, columns 2 to 4: 3 rows by 2 columns
  const id = model.newChart(0, anchor(2, 2, 3, 2), columnChart("Visitors"));

  const { worksheet, chartsLayer } = await renderWorksheet(model, {
    width: 430,
    height: 230,
  });
  // The layer covers the cells area
  expect(rect(chartsLayer)).toEqual({
    left: headerColumnWidth,
    top: headerRowHeight,
    width: 430 - headerColumnWidth,
    height: 230 - headerRowHeight,
  });
  expect(chartElements(chartsLayer).length).toBe(1);
  const chart = chartElements(chartsLayer)[0];
  expect(chart.className).toBe("ic-worksheet-chart");
  expect(chart.dataset.chartId).toBe(`${id}`);
  expect(rect(chart)).toEqual({
    left: COLUMN_WIDTH,
    top: ROW_HEIGHT,
    width: 2 * COLUMN_WIDTH,
    height: 3 * ROW_HEIGHT,
  });
  expect(chart.innerHTML).toMatch(/^<svg [^>]*width="180" height="75"/);
  expect(chart.innerHTML).toContain(">Visitors</text>");

  // Gone from the sheet, gone from the layer
  model.deleteChart(0, id);
  worksheet.renderSheet();
  expect(chartElements(chartsLayer).length).toBe(0);
  model.undo();
  worksheet.renderSheet();
  expect(chartElements(chartsLayer).length).toBe(1);
});

test("charts outside the viewport are not rendered", async () => {
  const model = await newModel();
  // Far below the 230px tall viewport
  model.newChart(0, anchor(100, 1, 10, 4), columnChart("Far away"));
  const { chartsLayer } = await renderWorksheet(model);
  expect(chartElements(chartsLayer).length).toBe(0);
});

test("the selected chart gets a frame with handles over it", async () => {
  const model = await newModel();
  const id = model.newChart(0, anchor(2, 2, 3, 2), columnChart("Selected"));
  const other = model.newChart(0, anchor(2, 5, 3, 2), columnChart("Other"));
  const workbookState = new WorkbookState();
  workbookState.setSelectedChart({ sheet: 0, id });
  const { worksheet, chartsLayer } = await renderWorksheet(model, {
    workbookState,
  });
  const children = Array.from(chartsLayer.children) as HTMLDivElement[];
  const frame = children.find((child) =>
    child.className.includes("ic-worksheet-chart-frame"),
  );
  const selected = chartElements(chartsLayer).find(
    (child) => child.dataset.chartId === `${id}`,
  );
  const otherElement = chartElements(chartsLayer).find(
    (child) => child.dataset.chartId === `${other}`,
  );
  if (!frame || !selected || !otherElement) {
    throw new Error("the layer is missing an element");
  }
  // The frame sits exactly over the selected chart, last in the layer
  expect(selected.className).toContain("ic-worksheet-chart--selected");
  expect(otherElement.className).not.toContain("--selected");
  expect(rect(frame)).toEqual(rect(selected));
  expect(frame.style.display).toBe("");
  expect(children[children.length - 1]).toBe(frame);
  // Eight handles and the menu button
  expect(frame.children.length).toBe(9);
  expect(frame.dataset.chartId).toBe(`${id}`);

  // Deselected: the frame is hidden
  workbookState.setSelectedChart(null);
  worksheet.renderSheet();
  expect(frame.style.display).toBe("none");
  expect(selected.className).not.toContain("--selected");
});
