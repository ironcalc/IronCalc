export const ALLOWED_SCHEMES = [
  "http:",
  "https:",
  "ftp:",
  "ftps:",
  "file:",
  "mailto:",
];

/// Returns the target URL for the link or null if `input` is not a valid URL.
/// Scheme-less URLs like "www.example.com" get "https://" prepended.
export function normalizeUrl(input: string): string | null {
  const value = input.trim();
  if (!value) {
    return null;
  }
  try {
    const url = new URL(value);
    return ALLOWED_SCHEMES.includes(url.protocol) ? value : null;
  } catch {
    // not an absolute URL: accept domain-like values with an implicit https://
    if (/^[\w-]+(\.[\w-]+)+([/?#]\S*)?$/.test(value)) {
      return `https://${value}`;
    }
    return null;
  }
}

export function isValidEmail(input: string): boolean {
  // allow "someone@example.com?subject=Hello" (the query is part of the mailto URI)
  const address = input.split("?")[0];
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(address);
}

function decodeMailtoParam(value: string): string {
  try {
    return decodeURIComponent(value);
  } catch {
    // malformed percent-encoding: show it as-is
    return value;
  }
}

/// Splits a mailto URI (with or without the "mailto:" scheme) into the address,
/// the percent-decoded subject and body, and any other query parameters (kept
/// verbatim), e.g. "mailto:daniel@ironcalc.com?subject=hola%20que%20tal&cc=x" gives
/// { address: "daniel@ironcalc.com", subject: "hola que tal", body: "", otherParams: "cc=x" }.
export function parseMailto(target: string): {
  address: string;
  subject: string;
  body: string;
  otherParams: string;
} {
  const rest = target.startsWith("mailto:")
    ? target.slice("mailto:".length)
    : target;
  const queryStart = rest.indexOf("?");
  if (queryStart === -1) {
    return { address: rest, subject: "", body: "", otherParams: "" };
  }
  const address = rest.slice(0, queryStart);
  let subject = "";
  let body = "";
  const others: string[] = [];
  for (const param of rest.slice(queryStart + 1).split("&")) {
    if (param.toLowerCase().startsWith("subject=")) {
      subject = decodeMailtoParam(param.slice("subject=".length));
    } else if (param.toLowerCase().startsWith("body=")) {
      body = decodeMailtoParam(param.slice("body=".length));
    } else if (param) {
      others.push(param);
    }
  }
  return { address, subject, body, otherParams: others.join("&") };
}

/// Builds a mailto URI from an address, a subject and a body (percent-encoded
/// here) and other query parameters (already encoded, kept verbatim).
export function buildMailto(
  address: string,
  subject: string,
  body: string,
  otherParams: string,
): string {
  const params: string[] = [];
  if (subject) {
    params.push(`subject=${encodeURIComponent(subject)}`);
  }
  if (body) {
    params.push(`body=${encodeURIComponent(body)}`);
  }
  if (otherParams) {
    params.push(otherParams);
  }
  if (params.length === 0) {
    return `mailto:${address}`;
  }
  return `mailto:${address}?${params.join("&")}`;
}

/// A single cell ("B5") or a range of cells ("A1:B5"), with optional dollars
export const CELL_REFERENCE_REGEX =
  /^\$?[A-Za-z]{1,3}\$?[0-9]{1,7}(:\$?[A-Za-z]{1,3}\$?[0-9]{1,7})?$/;
export const DEFINED_NAME_REGEX = /^[A-Za-z_][A-Za-z0-9_.]*$/;

/// Splits an internal link location ("Sheet1!A30", "'My Sheet'!A30" or a
/// defined name) into its sheet name and cell reference parts.
export function parseLocation(location: string): {
  sheetName: string;
  cellRef: string;
} {
  const separator = location.lastIndexOf("!");
  if (separator === -1) {
    return { sheetName: "", cellRef: location };
  }
  return {
    sheetName: location.slice(0, separator).replace(/^'(.*)'$/, "$1"),
    cellRef: location.slice(separator + 1),
  };
}
