/**
 * Creates an iframe running the IronCalc app and initializes it via postMessage.
 *
 * The parent and iframe use a small handshake to avoid race conditions:
 *
 * 1. Parent loads iframe and sends "ironcalc:init:v1" on load.
 * 2. Iframe finishes setup and replies with "ironcalc:ready:v1".
 * 3. Parent verifies origin and sends either a workbook or an empty model.
 *
 * The first message uses "*" since the origin is not known yet. After the
 * "ready" response, communication is restricted to the iframe’s origin.
 */

type MountTarget = string | HTMLElement;

type IronCalcEmbedOptions = {
  src?: string;
  title?: string;
  loading?: "eager" | "lazy";
  style?: Partial<CSSStyleDeclaration>;
  workbookBytes?: ArrayBuffer;
};

type IronCalcMessage =
  | { type: "ironcalc:init:v1" }
  | { type: "ironcalc:ready:v1" }
  | { type: "ironcalc:load-workbook:v1"; workbookBytes: ArrayBuffer }
  | { type: "ironcalc:load-empty:v1" };

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

  // The iframe registers its message listener only after WASM finishes loading,
  // which happens after onload fires. Retry until we get ironcalc:ready:v1.
  let retryInterval: ReturnType<typeof setInterval> | null = null;

  iframe.onload = () => {
    iframe.contentWindow?.postMessage({ type: "ironcalc:init:v1" }, "*");
    retryInterval = setInterval(() => {
      iframe.contentWindow?.postMessage({ type: "ironcalc:init:v1" }, "*");
    }, 200);
  };

  function cleanup() {
    if (retryInterval !== null) {
      clearInterval(retryInterval);
      retryInterval = null;
    }
    clearTimeout(timeoutId);
    window.removeEventListener("message", onMessage);
  }

  // Guard against the iframe never posting ironcalc-ready (e.g. network
  // failure), which would otherwise leave this listener registered indefinitely.
  const timeoutId = setTimeout(() => cleanup(), 30_000);

  function onMessage(event: MessageEvent<IronCalcMessage>) {
    if (!iframe.contentWindow) {
      return;
    }
    if (event.source !== iframe.contentWindow) {
      return;
    }
    if (event.origin !== targetOrigin) {
      return;
    }

    const data = event.data;
    if (!data || data.type !== "ironcalc:ready:v1") {
      return;
    }

    cleanup();

    if (options.workbookBytes) {
      iframe.contentWindow.postMessage(
        {
          type: "ironcalc:load-workbook:v1",
          workbookBytes: options.workbookBytes,
        },
        targetOrigin,
        [options.workbookBytes],
      );
    } else {
      iframe.contentWindow.postMessage(
        { type: "ironcalc:load-empty:v1" },
        targetOrigin,
      );
    }
  }

  window.addEventListener("message", onMessage);
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
