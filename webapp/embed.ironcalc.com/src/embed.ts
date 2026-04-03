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

function toUint8Array(bytes: Uint8Array | ArrayBuffer): Uint8Array {
  return bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
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

    const workbookBytes = options.workbookBytes;
    if (workbookBytes) {
      iframe.contentWindow?.postMessage(
        {
          type: "ironcalc-load-workbook",
          workbookBytes: toUint8Array(workbookBytes),
        },
        targetOrigin,
      );
    } else {
      iframe.contentWindow?.postMessage(
        {
          type: "ironcalc-load-empty-workbook",
        },
        targetOrigin,
      );
    }
    window.removeEventListener("message", onMessage);
  }

  window.addEventListener("message", onMessage);

  element.replaceChildren(iframe);

  return iframe;
}

const api: IronCalcEmbedApi = { mount };

window.IronCalcEmbed = api;

export { mount };
