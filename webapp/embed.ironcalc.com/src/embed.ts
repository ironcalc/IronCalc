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

/** "system" follows the visitor's OS preference (prefers-color-scheme). */
type IronCalcEmbedTheme = "light" | "dark" | "system";

type IronCalcEmbedOptions = {
  src?: string;
  title?: string;
  loading?: "eager" | "lazy";
  style?: Partial<CSSStyleDeclaration>;
  workbookBytes?: ArrayBuffer;
  /** Colour scheme of the spreadsheet. Defaults to "light". */
  theme?: IronCalcEmbedTheme;
};

type IronCalcMessage =
  | { type: "ironcalc:init:v1" }
  | { type: "ironcalc:ready:v1" }
  | { type: "ironcalc:load-workbook:v1"; workbookBytes: ArrayBuffer }
  | { type: "ironcalc:load-empty:v1" }
  | { type: "ironcalc:set-theme:v1"; theme: IronCalcEmbedTheme };

type IronCalcEmbedApi = {
  mount: (
    target: MountTarget,
    options?: IronCalcEmbedOptions,
  ) => HTMLIFrameElement;
  setTheme: (iframe: HTMLIFrameElement, theme: IronCalcEmbedTheme) => void;
};

type EmbedState = {
  theme: IronCalcEmbedTheme;
  targetOrigin: string;
  // Set once the iframe answered ironcalc:ready:v1; before that, messages
  // would be lost so the theme is only recorded and sent on ready.
  ready: boolean;
};

const embeds = new WeakMap<HTMLIFrameElement, EmbedState>();

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

  const theme = options.theme ?? "light";

  const iframe = document.createElement("iframe");
  const iframeUrl = new URL(
    options.src ?? "https://embed.ironcalc.com/",
    window.location.href,
  );
  // Lets the iframe paint the right colours while WASM is still loading.
  iframeUrl.searchParams.set("theme", theme);
  iframe.src = iframeUrl.href;
  iframe.title = options.title ?? "IronCalc spreadsheet";
  iframe.loading = options.loading ?? "lazy";

  if (options.style) {
    Object.assign(iframe.style, options.style);
  }

  const targetOrigin = iframeUrl.origin;
  const state: EmbedState = { theme, targetOrigin, ready: false };
  embeds.set(iframe, state);

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
    if (data?.type !== "ironcalc:ready:v1") {
      return;
    }

    cleanup();
    state.ready = true;

    // The theme may have changed via setTheme() while WASM was loading.
    postTheme(iframe, state);

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

function postTheme(iframe: HTMLIFrameElement, state: EmbedState) {
  iframe.contentWindow?.postMessage(
    { type: "ironcalc:set-theme:v1", theme: state.theme },
    state.targetOrigin,
  );
}

/** Switches the colour scheme of an iframe returned by mount(). */
function setTheme(iframe: HTMLIFrameElement, theme: IronCalcEmbedTheme): void {
  const state = embeds.get(iframe);
  if (!state) {
    console.warn("IronCalcEmbed: setTheme called on an unknown iframe");
    return;
  }
  state.theme = theme;
  if (state.ready) {
    postTheme(iframe, state);
  }
}

const api: IronCalcEmbedApi = { mount, setTheme };

if (window.IronCalcEmbed) {
  console.warn(
    "IronCalcEmbed: already defined — remove duplicate script tags to avoid this.",
  );
} else {
  window.IronCalcEmbed = api;
}

export type { IronCalcEmbedOptions, IronCalcEmbedTheme };
export { mount, setTheme };
