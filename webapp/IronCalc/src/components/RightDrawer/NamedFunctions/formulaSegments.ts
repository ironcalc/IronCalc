import { getTokens } from "@ironcalc/wasm";

/** A piece of a named-function formula: plain text or an argument chip */
export interface FormulaSegment {
  text: string;
  kind: "text" | "argument" | "unknown";
}

function sliceScalars(text: string, start: number, end: number): string {
  return Array.from(text).slice(start, end).join("");
}

/**
 * Splits the body of a named function into plain text and chips: a name that
 * is one of the `parameters` is an argument chip, one in `unknownNames` (as
 * reported by the engine's check) is an unknown chip. Everything else, the
 * leading `=` included, is text. Matching is case-sensitive, like the
 * evaluation.
 */
export function getFormulaSegments(
  formula: string,
  parameters: string[],
  unknownNames: string[] = [],
): FormulaSegment[] {
  const segments: FormulaSegment[] = [];
  const pushText = (text: string) => {
    if (text === "") {
      return;
    }
    const last = segments[segments.length - 1];
    if (last && last.kind === "text") {
      last.text += text;
    } else {
      segments.push({ text, kind: "text" });
    }
  };
  const hasEquals = formula.startsWith("=");
  const body = hasEquals ? formula.slice(1) : formula;
  if (hasEquals) {
    pushText("=");
  }
  let tokens: ReturnType<typeof getTokens>;
  try {
    tokens = getTokens(body);
  } catch {
    pushText(body);
    return segments;
  }
  let position = 0;
  for (const { token, start, end } of tokens) {
    if (start > position) {
      pushText(sliceScalars(body, position, start));
    }
    // A token's span starts right after the previous token, so it may begin
    // with whitespace: that is text, the name is what is left
    const raw = sliceScalars(body, start, end);
    const text = raw.trimStart();
    pushText(raw.slice(0, raw.length - text.length));
    if (typeof token === "object" && "Ident" in token) {
      if (parameters.includes(text)) {
        segments.push({ text, kind: "argument" });
      } else if (unknownNames.includes(text)) {
        segments.push({ text, kind: "unknown" });
      } else {
        pushText(text);
      }
    } else {
      pushText(text);
    }
    position = end;
  }
  pushText(sliceScalars(body, position, Array.from(body).length));
  return segments;
}

/** The first name of the form `base`, `base_1`, `base_2`... not in `taken` */
export function firstFreeName(base: string, taken: string[]): string {
  const upper = taken.map((name) => name.toUpperCase());
  if (!upper.includes(base.toUpperCase())) {
    return base;
  }
  let counter = 1;
  while (upper.includes(`${base}_${counter}`.toUpperCase())) {
    counter += 1;
  }
  return `${base}_${counter}`;
}
