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
