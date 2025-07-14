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
  return new Uint8Array(
    await (await fetch(`/api/model/${modelHash}`)).arrayBuffer(),
  );
}

export async function get_documentation_model(
  filename: string,
): Promise<Uint8Array> {
  return new Uint8Array(
    await (await fetch(`/models/${filename}.ic`)).arrayBuffer(),
  );
}

export async function downloadModel(bytes: Uint8Array, fileName: string) {
  const sanitizedFileName = sanitizeFileName(fileName);
  const response = await fetch("/api/download", {
    method: "POST",
    headers: {
      "Content-Type": "application/octet-stream",
    },
    body: bytes,
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

export async function shareModel(bytes: Uint8Array): Promise<string> {
  const response = await fetch("/api/share", {
    method: "POST",
    headers: {
      "Content-Type": "application/octet-stream",
    },
    body: bytes,
  });
  if (!response.ok) {
    throw new Error("Network response was not ok");
  }
  return await response.text();
}
