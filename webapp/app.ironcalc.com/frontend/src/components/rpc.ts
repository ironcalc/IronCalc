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
