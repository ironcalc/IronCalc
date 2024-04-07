import { expect, test } from "vitest";
import { isNavigationKey } from "../util";

test("checks arrow left is a navigation key", () => {
  expect(isNavigationKey("ArrowLeft")).toBe(true);
  expect(isNavigationKey("Arrow")).toBe(false);
});
