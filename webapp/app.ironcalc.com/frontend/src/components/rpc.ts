const MAX_FILENAME_LENGTH = 100;

function sanitizeFileName(name: string): string {
  const normalized = name.normalize("NFKC");

  const safe = [...normalized]
    .map((char) => {
      const code = char.charCodeAt(0);
      // Remove control chars and filesystem-unsafe chars
      if (
        code <= 0x1f || // ASCII control
        code === 0x7f || // DEL
        ["<", ">", ":", '"', "/", "\\", "|", "?", "*"].includes(char)
      ) {
        return "_";
      }
      return char;
    })
    .join("");

  return safe.slice(0, MAX_FILENAME_LENGTH).trim();
}

export async function uploadFile(
  arrayBuffer: ArrayBuffer,
  fileName: string,
): Promise<Blob> {
  // Fetch request to upload the file
  const response = await fetch(`/api/upload/${fileName}`, {
    method: "POST",
    headers: {
      "Content-Type": "application/octet-stream",
      "Content-Disposition": `attachment; filename="${fileName}"`,
    },
    body: arrayBuffer,
  });
  const blob = await response.blob();
  return blob;
}

export async function get_model(modelHash: string): Promise<Uint8Array> {
  const response = await fetch(`/api/model/${modelHash}`);
  if (!response.ok) {
    throw new Error(`Failed to load model: ${response.status}`);
  }
  return new Uint8Array(await response.arrayBuffer());
}

export async function get_documentation_model(
  filename: string,
): Promise<Uint8Array> {
  const response = await fetch(`/models/${filename}.ic`);
  if (!response.ok) {
    throw new Error(`Failed to load documentation model: ${response.status}`);
  }
  return new Uint8Array(await response.arrayBuffer());
}

export async function downloadModel(bytes: Uint8Array, fileName: string) {
  const sanitizedFileName = sanitizeFileName(fileName);
  // Create a real ArrayBuffer and copy the data
  const buffer = new ArrayBuffer(bytes.byteLength);
  new Uint8Array(buffer).set(bytes);
  const response = await fetch("/api/download", {
    method: "POST",
    headers: {
      "Content-Type": "application/octet-stream",
    },
    body: buffer,
  });
  if (!response.ok) {
    throw new Error("Network response was not ok");
  }
  const blob = await response.blob();

  // Create a link element and trigger a download
  const url = window.URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.style.display = "none";
  a.href = url;

  a.download = `${sanitizedFileName}.xlsx`;
  document.body.appendChild(a);
  a.click();

  // Clean up
  window.URL.revokeObjectURL(url);
  a.remove();
}

export interface PdfExportPayloadOptions {
  scope?: string;
  selection_range?: string;
  sheet_index?: number;
  paper_size?: string;
  custom_width_pt?: number;
  custom_height_pt?: number;
  orientation?: string;
  scale?: string;
  scale_percentage?: number;
  margins?: string;
  margin_top_pt?: number;
  margin_right_pt?: number;
  margin_bottom_pt?: number;
  margin_left_pt?: number;
  show_grid_lines?: boolean;
  show_headings?: boolean;
  page_order?: string;
  horizontal_alignment?: string;
  vertical_alignment?: string;
  page_numbers?: boolean;
  workbook_title?: boolean;
  sheet_name?: boolean;
  current_date?: boolean;
  current_time?: boolean;
  repeat_frozen_rows?: boolean;
  repeat_frozen_columns?: boolean;
}

export interface CustomFontPayload {
  name: string;
  data: string; // base64
}

export async function fetchModelPdfBlob(
  bytes: Uint8Array,
  options?: PdfExportPayloadOptions,
  customFonts?: CustomFontPayload[],
): Promise<Blob> {
  let response: Response;

  if (options || (customFonts && customFonts.length > 0)) {
    // Convert bytes to base64
    let binary = "";
    const len = bytes.byteLength;
    const chunkSize = 8192;
    for (let i = 0; i < len; i += chunkSize) {
      const chunk = bytes.subarray(i, Math.min(i + chunkSize, len));
      binary += String.fromCharCode.apply(null, chunk as unknown as number[]);
    }
    const base64Model = window.btoa(binary);

    response = await fetch("/api/download_pdf", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        model: base64Model,
        options,
        custom_fonts: customFonts,
      }),
    });
  } else {
    // Create a real ArrayBuffer and copy the data
    const buffer = new ArrayBuffer(bytes.byteLength);
    new Uint8Array(buffer).set(bytes);
    response = await fetch("/api/download_pdf", {
      method: "POST",
      headers: {
        "Content-Type": "application/octet-stream",
      },
      body: buffer,
    });
  }

  if (!response.ok) {
    const errorText = await response.text().catch(() => "");
    throw new Error(errorText || "Network response was not ok");
  }
  return await response.blob();
}

export function triggerBlobDownload(blob: Blob, fileName: string) {
  const sanitizedFileName = sanitizeFileName(fileName);
  const url = window.URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.style.display = "none";
  a.href = url;
  a.download = `${sanitizedFileName}.pdf`;
  document.body.appendChild(a);
  a.click();
  window.URL.revokeObjectURL(url);
  a.remove();
}

export async function downloadModelPdf(
  bytes: Uint8Array,
  fileName: string,
  options?: PdfExportPayloadOptions,
  customFonts?: CustomFontPayload[],
) {
  const blob = await fetchModelPdfBlob(bytes, options, customFonts);
  triggerBlobDownload(blob, fileName);
}

export async function shareModel(bytes: Uint8Array): Promise<string> {
  // Create a real ArrayBuffer and copy the data
  const buffer = new ArrayBuffer(bytes.byteLength);
  new Uint8Array(buffer).set(bytes);

  const response = await fetch("/api/share", {
    method: "POST",
    headers: {
      "Content-Type": "application/octet-stream",
    },
    body: buffer,
  });
  if (!response.ok) {
    throw new Error("Network response was not ok");
  }
  return await response.text();
}
