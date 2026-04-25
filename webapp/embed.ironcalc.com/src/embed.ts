type MountTarget = string | HTMLElement;

type IronCalcEmbedOptions = {
  src?: string;
  title?: string;
  loading?: "eager" | "lazy";
  style?: Partial<CSSStyleDeclaration>;
  workbookBytes?: Uint8Array | ArrayBuffer;
};

type IronCalcEmbedApi = {
  mount: (
    target: MountTarget,
    options?: IronCalcEmbedOptions,
  ) => HTMLIFrameElement;
};

declare global {
  interface Window {
    IronCalcEmbed?: IronCalcEmbedApi;
  }
}

function resolveTarget(target: MountTarget): HTMLElement {
  if (typeof target === "string") {
    const element = document.querySelector<HTMLElement>(target);
    if (!element) {
      throw new Error(
        `IronCalcEmbed: target not found for selector "${target}"`,
      );
    }
    return element;
  }
  return target;
}

function toTransferableBuffer(bytes: Uint8Array | ArrayBuffer): ArrayBuffer {
  if (bytes instanceof ArrayBuffer) {
    return bytes;
  }
  if (bytes.byteOffset === 0 && bytes.byteLength === bytes.buffer.byteLength) {
    return bytes.buffer;
  }
  // Uint8Array sub-view: copy only the relevant slice into a standalone buffer.
  return bytes.buffer.slice(
    bytes.byteOffset,
    bytes.byteOffset + bytes.byteLength,
  );
}

function mount(
  target: MountTarget,
  options: IronCalcEmbedOptions = {},
): HTMLIFrameElement {
  const element = resolveTarget(target);

  const iframe = document.createElement("iframe");
  iframe.src = options.src ?? "https://embed.ironcalc.com/";
  iframe.title = options.title ?? "IronCalc spreadsheet";
  iframe.loading = options.loading ?? "lazy";

  if (options.style) {
    Object.assign(iframe.style, options.style);
  }

  const iframeUrl = new URL(iframe.src, window.location.href);
  const targetOrigin = iframeUrl.origin;

  const controller = new AbortController();
  const { signal } = controller;

  // Guard against the iframe never posting ironcalc-ready (e.g. network
  // failure), which would otherwise leave this listener attached indefinitely.
  const timeoutId = setTimeout(() => controller.abort(), 30_000);

  function onMessage(event: MessageEvent) {
    if (event.source !== iframe.contentWindow) {
      return;
    }

    if (event.origin !== targetOrigin) {
      return;
    }

    const data = event.data;
    if (!data || data.type !== "ironcalc-ready") {
      return;
    }

    clearTimeout(timeoutId);
    controller.abort();

    const workbookBytes = options.workbookBytes;
    if (workbookBytes) {
      const buffer = toTransferableBuffer(workbookBytes);
      iframe.contentWindow?.postMessage(
        { type: "ironcalc-load-workbook", workbookBytes: buffer },
        targetOrigin,
        [buffer],
      );
    } else {
      iframe.contentWindow?.postMessage(
        {
          type: "ironcalc-load-empty-workbook",
        },
        targetOrigin,
      );
    }
  }

  // { signal } ensures the listener is removed both after a successful load
  // (controller.abort() above) and on timeout.
  window.addEventListener("message", onMessage, { signal });

  element.replaceChildren(iframe);

  return iframe;
}

const api: IronCalcEmbedApi = { mount };

if (window.IronCalcEmbed) {
  console.warn(
    "IronCalcEmbed: already defined — remove duplicate script tags to avoid this.",
  );
} else {
  window.IronCalcEmbed = api;
}

export { mount };
