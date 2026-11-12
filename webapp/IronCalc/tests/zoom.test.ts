import { expect, test } from "vitest";
import {
  clampZoom,
  MAX_ZOOM,
  MIN_ZOOM,
  WorkbookState,
  zoomInLevel,
  zoomOutLevel,
} from "../src/components/workbookState";

test("zoom levels step by 10%, snapping to the next multiple", () => {
  expect(zoomInLevel(100)).toBe(110);
  expect(zoomOutLevel(100)).toBe(90);
  // from a level between steps (a pinch), the next multiple of 10
  expect(zoomInLevel(105)).toBe(110);
  expect(zoomOutLevel(105)).toBe(100);
  expect(zoomInLevel(109.99)).toBe(110);
  expect(zoomOutLevel(100.01)).toBe(100);
});

test("the zoom stays within Excel's range", () => {
  expect(zoomInLevel(MAX_ZOOM)).toBe(MAX_ZOOM);
  expect(zoomOutLevel(MIN_ZOOM)).toBe(MIN_ZOOM);
  expect(clampZoom(5)).toBe(MIN_ZOOM);
  expect(clampZoom(1000)).toBe(MAX_ZOOM);
  expect(clampZoom(150)).toBe(150);
});

test("a workbook starts at 100% and clamps what it is set to", () => {
  const state = new WorkbookState();
  expect(state.getZoom()).toBe(100);
  state.setZoom(150);
  expect(state.getZoom()).toBe(150);
  state.setZoom(0);
  expect(state.getZoom()).toBe(MIN_ZOOM);
});
