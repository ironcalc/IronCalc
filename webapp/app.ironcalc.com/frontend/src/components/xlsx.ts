import type { Model } from "@ironcalc/workbook";

const MAX_FILENAME_LENGTH = 100;

const XLSX_MIME_TYPE =
  "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

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

// Exports the workbook to xlsx in the browser (through @ironcalc/wasm-xlsx)
// and triggers a download of the file.
export function downloadXlsx(model: Model, fileName: string) {
  const bytes = model.toXlsx();
  // Create a real ArrayBuffer and copy the data
  const buffer = new ArrayBuffer(bytes.byteLength);
  new Uint8Array(buffer).set(bytes);
  const blob = new Blob([buffer], { type: XLSX_MIME_TYPE });

  // Create a link element and trigger a download
  const url = window.URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.style.display = "none";
  a.href = url;

  a.download = `${sanitizeFileName(fileName)}.xlsx`;
  document.body.appendChild(a);
  a.click();

  // Clean up
  window.URL.revokeObjectURL(url);
  a.remove();
}
