import { expect, test } from "vitest";
import { docsUrl } from "../src/components/FormulaHelper/formulaCompletion";
import functions from "../src/components/FormulaHelper/functions.json";

// Category numbers come from functions.json: 4 = financial, 9 = statistical.

test("a single-word function links to its page with no anchor", () => {
  expect(docsUrl("ACCRINT", 4)).toBe(
    "https://docs.ironcalc.com/functions/financial/accrint.html",
  );
});

test("a dotted function keeps the dot in the file and dashes the anchor", () => {
  expect(docsUrl("T.TEST", 9)).toBe(
    "https://docs.ironcalc.com/functions/statistical/t.test.html#t-test",
  );
});

test("an unknown category (no docs page) returns null", () => {
  // Category 11 is used by legacy functions with no dedicated docs page.
  expect(docsUrl("TTEST", 11)).toBeNull();
});

// MAX and MIN are statistical, not math (their docs live under statistical/).
// Guard the functions.json category fix so the links don't 404 again.
test.each(["MAX", "MIN"])("%s links to the statistical docs page", (name) => {
  const category = (functions as Record<string, { category: number }>)[
    name.toLowerCase()
  ].category;
  expect(docsUrl(name, category)).toBe(
    `https://docs.ironcalc.com/functions/statistical/${name.toLowerCase()}.html`,
  );
});
