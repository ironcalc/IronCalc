#!/usr/bin/env node
/**
 * i18n lint script
 *
 * Checks:
 *   1. All locale files have the same keys as the reference locale (en_us.json)
 *   2. No locale keys are unused (defined but never referenced in source)
 *   3. No locale keys are missing (referenced in source but not defined)
 *
 * Dynamic keys: some keys are built at runtime via template literals, e.g.
 *   t(`conditional_formatting.color_scale_type_${v}`)
 * The script detects these automatically and exempts keys matching the prefix.
 * Add explicit entries to DYNAMIC_PREFIXES below for cases the auto-detection
 * cannot pick up (e.g. the prefix itself is dynamic).
 */

import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

// Explicit dynamic-prefix allowlist (auto-detected prefixes are also included)
const DYNAMIC_PREFIXES = [];

const ROOT = process.cwd();
const LOCALE_DIR = join(ROOT, "src/locale");
const SRC_DIR = join(ROOT, "src");
const REFERENCE_LOCALE = "en_us.json";

// ---- helpers ----------------------------------------------------------------

function flattenKeys(obj, prefix = "") {
  const keys = [];
  for (const [k, v] of Object.entries(obj)) {
    const full = prefix ? `${prefix}.${k}` : k;
    if (v !== null && typeof v === "object") {
      keys.push(...flattenKeys(v, full));
    } else {
      keys.push(full);
    }
  }
  return keys;
}

function* walkTs(dir) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const fullPath = join(dir, entry.name);
    if (entry.isDirectory()) {
      yield* walkTs(fullPath);
    } else if (entry.name.endsWith(".ts") || entry.name.endsWith(".tsx")) {
      yield fullPath;
    }
  }
}

// Primary: t("key") / t('key') / t("key", {...}) / t(\n  "key"\n)
// \s* handles multiline calls; no closing \) so options args don't break it.
// Negative lookbehind avoids false positives like document.createElement("div").
const STATIC_RE = /(?<![a-zA-Z])t\(\s*["']([a-z][^"']*)["']/g;

// Secondary: any quoted string literal that looks like a locale key.
// This catches indirect references like `labelKey: "toolbar.borders.all"`
// later passed to t(obj.labelKey). We intersect against the actual key set
// so there are no false positives from unrelated dot-separated strings.
const LOCALE_KEY_RE = /["']([a-z][a-z0-9_]*(?:\.[a-z0-9_]+)+)["']/g;

// Dynamic template literals: t(`prefix${expr}`) — capture the static prefix.
// \s* handles multiline calls like t(\n  `prefix${expr}`\n).
const DYNAMIC_RE = /(?<![a-zA-Z])t\(\s*`([^`$]*)\$\{[^}]+\}[^`]*`/g;

function extractUsedKeys(srcDir) {
  const staticKeys = new Set();
  const detectedPrefixes = new Set();

  for (const file of walkTs(srcDir)) {
    const content = readFileSync(file, "utf8");

    for (const m of content.matchAll(STATIC_RE)) {
      staticKeys.add(m[1]);
    }
    for (const m of content.matchAll(LOCALE_KEY_RE)) {
      staticKeys.add(m[1]); // intersected with locale keys later
    }
    for (const m of content.matchAll(DYNAMIC_RE)) {
      if (m[1]) detectedPrefixes.add(m[1]);
    }
  }

  return { staticKeys, detectedPrefixes };
}

// ---- main -------------------------------------------------------------------

let errors = 0;

function fail(msg) {
  console.error(msg);
  errors++;
}

// Load locale files
const localeFiles = readdirSync(LOCALE_DIR)
  .filter((f) => f.endsWith(".json"))
  .sort();

const locales = {};
for (const file of localeFiles) {
  const raw = readFileSync(join(LOCALE_DIR, file), "utf8");
  locales[file] = flattenKeys(JSON.parse(raw));
}

if (!locales[REFERENCE_LOCALE]) {
  console.error(`Reference locale not found: ${REFERENCE_LOCALE}`);
  process.exit(1);
}

const referenceKeys = new Set(locales[REFERENCE_LOCALE]);

// 1. Key consistency across locales
for (const [file, keys] of Object.entries(locales)) {
  if (file === REFERENCE_LOCALE) continue;
  const keySet = new Set(keys);
  const missing = [...referenceKeys].filter((k) => !keySet.has(k)).sort();
  const extra = keys.filter((k) => !referenceKeys.has(k)).sort();
  if (missing.length) {
    fail(
      `[${file}] ${missing.length} key(s) missing from reference:\n${missing.map((k) => `  - ${k}`).join("\n")}`,
    );
  }
  if (extra.length) {
    fail(
      `[${file}] ${extra.length} extra key(s) not in reference:\n${extra.map((k) => `  + ${k}`).join("\n")}`,
    );
  }
}

// 2. Used vs defined
const { staticKeys, detectedPrefixes } = extractUsedKeys(SRC_DIR);

// Keep only source strings that are actually locale keys (safe intersection)
const usedLocaleKeys = new Set(
  [...staticKeys].filter((k) => referenceKeys.has(k)),
);

const allDynamicPrefixes = [
  ...new Set([...DYNAMIC_PREFIXES, ...detectedPrefixes]),
];

if (allDynamicPrefixes.length) {
  console.log(
    `Dynamic key prefixes (exempted from unused check): ${allDynamicPrefixes.map((p) => `"${p}"`).join(", ")}`,
  );
}

function isCoveredByDynamic(key) {
  return allDynamicPrefixes.some((p) => key.startsWith(p));
}

const unusedKeys = [...referenceKeys]
  .filter((k) => !usedLocaleKeys.has(k) && !isCoveredByDynamic(k))
  .sort();

// Missing keys: used in t("key") but not in locale. The broad LOCALE_KEY_RE
// scan catches many strings that aren't translation keys, so we only report
// keys from the narrow STATIC_RE that aren't in the locale at all.
const narrowStaticKeys = new Set();
for (const file of walkTs(SRC_DIR)) {
  const content = readFileSync(file, "utf8");
  for (const m of content.matchAll(STATIC_RE)) {
    narrowStaticKeys.add(m[1]);
  }
}
const missingKeys = [...narrowStaticKeys]
  .filter((k) => !referenceKeys.has(k))
  .sort();

if (unusedKeys.length) {
  fail(
    `${unusedKeys.length} locale key(s) defined but never used:\n${unusedKeys.map((k) => `  - ${k}`).join("\n")}`,
  );
}

if (missingKeys.length) {
  fail(
    `${missingKeys.length} key(s) used in source but missing from locale:\n${missingKeys.map((k) => `  + ${k}`).join("\n")}`,
  );
}

// Summary
if (errors === 0) {
  const keyCount = referenceKeys.size;
  const localeCount = localeFiles.length;
  console.log(`✓ i18n OK — ${keyCount} keys across ${localeCount} locales`);
} else {
  process.exit(1);
}
