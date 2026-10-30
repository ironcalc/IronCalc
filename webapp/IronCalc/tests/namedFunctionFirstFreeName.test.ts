import { expect, test } from "vitest";
import { firstFreeName } from "../src/components/RightDrawer/NamedFunctions/firstFreeName";

test("first free name appends a counter", () => {
  expect(firstFreeName("NEW_FUNCTION", [])).toBe("NEW_FUNCTION");
  expect(firstFreeName("NEW_FUNCTION", ["new_function"])).toBe(
    "NEW_FUNCTION_1",
  );
  expect(firstFreeName("HOLA", ["HOLA", "HOLA_1", "Other"])).toBe("HOLA_2");
});
