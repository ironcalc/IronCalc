// Standard 14 PDF fonts are built into PDF engines; custom web fonts must be fetched and embedded.
const BUNDLED_FONTS = new Set<string>();

// Metadata JSON URL (small file, cacheable via CDN)
const GOOGLE_FONTS_METADATA_URL =
  "https://cdn.jsdelivr.net/npm/google-font-metadata@6/data/google-fonts-v1.json";

const TRUSTED_FONT_DOMAINS = new Set([
  "fonts.gstatic.com",
  "fonts.googleapis.com",
  "cdn.jsdelivr.net",
]);

function isTrustedFontUrl(urlStr: string): boolean {
  try {
    const url = new URL(urlStr);
    return url.protocol === "https:" && TRUSTED_FONT_DOMAINS.has(url.hostname);
  } catch {
    return false;
  }
}

interface GoogleFontV1Entry {
  family: string;
  id: string;
  weights: number[];
  styles: string[];
  defSubset: string;
  variants: Record<
    string,
    Record<
      string,
      Record<string, { url: { truetype?: string; woff2?: string } }>
    >
  >;
}

// In-memory cache for the metadata JSON (fetched once per session)
let metadataCache: Record<string, GoogleFontV1Entry> | null = null;
let metadataFetchPromise: Promise<Record<
  string,
  GoogleFontV1Entry
> | null> | null = null;

/**
 * Fetch and cache the google-font-metadata v1 JSON.
 */
export async function getGoogleFontMetadata(): Promise<Record<
  string,
  GoogleFontV1Entry
> | null> {
  if (metadataCache) return metadataCache;
  if (metadataFetchPromise) return metadataFetchPromise;

  metadataFetchPromise = (async () => {
    try {
      const res = await fetch(GOOGLE_FONTS_METADATA_URL);
      if (!res.ok) return null;
      metadataCache = await res.json();
      return metadataCache;
    } catch {
      return null;
    } finally {
      metadataFetchPromise = null;
    }
  })();

  return metadataFetchPromise;
}

/**
 * Convert a display font family name (e.g. "Roboto Mono") to the metadata key ("roboto-mono").
 */
const fontFamilyToId = (family: string): string =>
  family.toLowerCase().replace(/\s+/g, "-");

/**
 * Look up the TTF (truetype) URL for a font face from cached metadata.
 */
function getTrueTypeUrl(
  metadata: Record<string, GoogleFontV1Entry>,
  family: string,
  weight = 400,
  style = "normal",
): string | undefined {
  const id = fontFamilyToId(family);
  const font = metadata[id];
  if (!font) return undefined;

  const weightVariants = font.variants[String(weight)];
  if (!weightVariants) {
    const firstWeight = Object.keys(font.variants)[0];
    if (!firstWeight) return undefined;
    const fallback = font.variants[firstWeight];
    const styleVar = fallback?.[style] ?? Object.values(fallback ?? {})[0];
    const subset =
      styleVar?.[font.defSubset] ?? Object.values(styleVar ?? {})[0];
    return subset?.url?.truetype;
  }

  const styleVariants =
    weightVariants[style] ?? Object.values(weightVariants)[0];
  if (!styleVariants) return undefined;

  const subset =
    styleVariants[font.defSubset] ?? Object.values(styleVariants)[0];
  return subset?.url?.truetype;
}

export interface FontFaceRequest {
  family: string;
  weight: number;
  style: "normal" | "italic";
}

export interface LoadedFontFace {
  family: string;
  weight: number;
  style: "normal" | "italic";
  bytes: Uint8Array;
}

// In-memory cache for downloaded font face bytes: `${family}:${weight}:${style}` -> bytes
const fontBytesCache = new Map<string, Uint8Array>();

function fontFaceKey(family: string, weight: number, style: string): string {
  return `${family.toLowerCase().trim()}:${weight}:${style}`;
}

/**
 * Fetch font face data (family, weight, style) from Google Fonts CDN.
 */
export async function fetchFontsForFaces(
  faces: readonly FontFaceRequest[],
): Promise<{ fonts: LoadedFontFace[]; warnings: string[] }> {
  const fonts: LoadedFontFace[] = [];
  const warnings: string[] = [];

  const seen = new Set<string>();
  const uniqueFaces: FontFaceRequest[] = [];
  for (const f of faces) {
    if (BUNDLED_FONTS.has(f.family)) continue;
    const key = fontFaceKey(f.family, f.weight, f.style);
    if (!seen.has(key)) {
      seen.add(key);
      uniqueFaces.push(f);
    }
  }

  const toFetch: FontFaceRequest[] = [];
  for (const f of uniqueFaces) {
    const cached = fontBytesCache.get(fontFaceKey(f.family, f.weight, f.style));
    if (cached) {
      fonts.push({
        family: f.family,
        weight: f.weight,
        style: f.style,
        bytes: cached,
      });
    } else {
      toFetch.push(f);
    }
  }

  if (toFetch.length === 0) return { fonts, warnings };

  const metadata = await getGoogleFontMetadata();
  if (!metadata) {
    warnings.push(
      "Could not load Google Fonts metadata. Text will use the default font.",
    );
    return { fonts, warnings };
  }

  const results = await Promise.allSettled(
    toFetch.map(async (f) => {
      const ttfUrl = getTrueTypeUrl(metadata, f.family, f.weight, f.style);
      if (!ttfUrl || !isTrustedFontUrl(ttfUrl))
        return { ...f, bytes: null as Uint8Array | null };
      try {
        const res = await fetch(ttfUrl);
        if (!res.ok) return { ...f, bytes: null };
        const buf = await res.arrayBuffer();
        return {
          ...f,
          bytes: buf.byteLength > 1024 ? new Uint8Array(buf) : null,
        };
      } catch {
        return { ...f, bytes: null };
      }
    }),
  );

  for (const result of results) {
    if (result.status === "fulfilled") {
      const { family, weight, style, bytes } = result.value;
      if (bytes) {
        fontBytesCache.set(fontFaceKey(family, weight, style), bytes);
        fonts.push({ family, weight, style, bytes });
      } else {
        warnings.push(
          `Font "${family}" (${weight} ${style}) could not be fetched.`,
        );
      }
    } else {
      warnings.push("A font fetch failed unexpectedly.");
    }
  }

  return { fonts, warnings };
}

/**
 * Backward-compatible helper to fetch font data for specified font families.
 */
export async function fetchFontsForFamilies(
  families: readonly string[],
): Promise<{ fontMap: Map<string, Uint8Array>; warnings: string[] }> {
  const faces: FontFaceRequest[] = [];
  for (const f of families) {
    faces.push({ family: f, weight: 400, style: "normal" });
    faces.push({ family: f, weight: 700, style: "normal" });
  }
  const { fonts, warnings } = await fetchFontsForFaces(faces);
  const fontMap = new Map<string, Uint8Array>();
  for (const f of fonts) {
    fontMap.set(f.family, f.bytes);
  }
  return { fontMap, warnings };
}

interface ModelWithFonts {
  getFontDescriptors?: () => unknown;
  getFontFamilies?: () => unknown;
}

/**
 * Collect all required font faces (family, weight, style) from the model.
 */
export function collectFontFacesFromModel(
  model: ModelWithFonts | unknown,
): FontFaceRequest[] {
  const requests: FontFaceRequest[] = [];
  const seen = new Set<string>();

  const candidate = model as ModelWithFonts | undefined;
  if (typeof candidate?.getFontDescriptors === "function") {
    try {
      const descriptors = candidate.getFontDescriptors() as {
        family: string;
        bold: boolean;
        italic: boolean;
      }[];
      if (Array.isArray(descriptors)) {
        for (const d of descriptors) {
          const family = d.family?.trim();
          if (!family) continue;
          const weight = d.bold ? 700 : 400;
          const style = d.italic ? "italic" : "normal";
          const key = fontFaceKey(family, weight, style);
          if (!seen.has(key)) {
            seen.add(key);
            requests.push({ family, weight, style });
          }
        }
      }
    } catch (e) {
      console.warn("Could not get font descriptors from model:", e);
    }
  }

  if (requests.length === 0) {
    const families = collectFontFamiliesFromModel(model);
    for (const family of families) {
      for (const weight of [400, 700]) {
        for (const style of ["normal" as const, "italic" as const]) {
          const key = fontFaceKey(family, weight, style);
          if (!seen.has(key)) {
            seen.add(key);
            requests.push({ family, weight, style });
          }
        }
      }
    }
  }

  return requests;
}

/**
 * Collect all font family names used in the workbook model.
 */
export function collectFontFamiliesFromModel(
  model: ModelWithFonts | unknown,
): string[] {
  const families = new Set<string>();

  const candidate = model as ModelWithFonts | undefined;
  if (typeof candidate?.getFontFamilies === "function") {
    try {
      const list = candidate.getFontFamilies();
      if (Array.isArray(list)) {
        for (const f of list) {
          if (typeof f === "string" && f.trim().length > 0) {
            families.add(f.trim());
          }
        }
      }
    } catch (e) {
      console.warn("Could not get font families from model:", e);
    }
  }

  if (families.size === 0) {
    families.add("Inter");
  }

  return [...families];
}

/**
 * Converts a Uint8Array into a standard base64 string.
 */
export function uint8ArrayToBase64(bytes: Uint8Array): string {
  let binary = "";
  const len = bytes.byteLength;
  const chunkSize = 8192;
  for (let i = 0; i < len; i += chunkSize) {
    const chunk = bytes.subarray(i, Math.min(i + chunkSize, len));
    binary += String.fromCharCode.apply(null, chunk as unknown as number[]);
  }
  return window.btoa(binary);
}
