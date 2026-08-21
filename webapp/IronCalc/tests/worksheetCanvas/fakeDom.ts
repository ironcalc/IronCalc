// A tiny stand-in for the DOM: just enough for WorksheetCanvas to run under
// vitest in plain node. Only the members the renderer actually touches are
// implemented, so any new DOM usage fails loudly and shows up here.
//
// The canvas itself is real (@napi-rs/canvas, a Skia rasterizer): only the
// DOM around it is faked.

export class FakeElement {
  tagName: string;
  parentElement: FakeElement | null = null;
  childNodes: FakeElement[] = [];
  classSet = new Set<string>();
  // The renderer only ever assigns style properties, it never reads them back
  style: Record<string, string> = {};
  textContent = "";

  constructor(tagName: string) {
    this.tagName = tagName;
  }

  get className(): string {
    return [...this.classSet].join(" ");
  }

  set className(value: string) {
    this.classSet = new Set(value.split(" ").filter(Boolean));
  }

  get classList(): {
    add: (name: string) => void;
    remove: (name: string) => void;
    contains: (name: string) => boolean;
  } {
    const classes = this.classSet;
    return {
      add: (name: string) => {
        classes.add(name);
      },
      remove: (name: string) => {
        classes.delete(name);
      },
      contains: (name: string) => classes.has(name),
    };
  }

  get children(): FakeElement[] {
    return this.childNodes;
  }

  appendChild(child: FakeElement): FakeElement {
    return this.insertBefore(child, null);
  }

  insertBefore(child: FakeElement, reference: FakeElement | null): FakeElement {
    child.remove();
    const index = reference ? this.childNodes.indexOf(reference) : -1;
    if (index === -1) {
      this.childNodes.push(child);
    } else {
      this.childNodes.splice(index, 0, child);
    }
    child.parentElement = this;
    return child;
  }

  remove(): void {
    const parent = this.parentElement;
    if (!parent) {
      return;
    }
    const index = parent.childNodes.indexOf(this);
    if (index !== -1) {
      parent.childNodes.splice(index, 1);
    }
    this.parentElement = null;
  }

  // The renderer only ever queries by a single class name (".foo")
  querySelectorAll(selector: string): FakeElement[] {
    const name = selector.slice(1);
    const found: FakeElement[] = [];
    const walk = (element: FakeElement): void => {
      for (const child of element.childNodes) {
        if (child.classSet.has(name)) {
          found.push(child);
        }
        walk(child);
      }
    };
    walk(this);
    return found;
  }

  closest(selector: string): FakeElement | null {
    const name = selector.slice(1);
    // biome-ignore lint/suspicious/noExplicitAny: `this` narrows the loop variable to FakeElement
    let element: FakeElement | null = this as any;
    while (element) {
      if (element.classSet.has(name)) {
        return element;
      }
      element = element.parentElement;
    }
    return null;
  }

  addEventListener(): void {}

  removeEventListener(): void {}

  getBoundingClientRect(): {
    x: number;
    y: number;
    top: number;
    left: number;
    right: number;
    bottom: number;
    width: number;
    height: number;
  } {
    return {
      x: 0,
      y: 0,
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      width: 6,
      height: 6,
    };
  }
}

export class FakeCanvasElement extends FakeElement {
  width = 0;
  height = 0;
  private context: unknown;

  constructor(context: unknown) {
    super("canvas");
    this.context = context;
  }

  getContext(): unknown {
    return this.context;
  }
}

// The theme WorksheetCanvas reads through getComputedStyle(".ic-root").
// Fixed values so screenshots never depend on the real CSS. The font stacks
// mirror theme.css; "Inter" is the app's own font, registered by the harness
// from fonts/*.woff2, so text rendering is identical on every machine.
export const testThemeVars: Record<string, string> = {
  "--palette-common-white": "#ffffff",
  "--palette-sheet-grid-color": "#dcdcdc",
  "--palette-sheet-default-cell-font-family":
    '"Inter", "Adjusted Arial Fallback", sans-serif',
  "--palette-primary-main": "#f2994a",
  "--palette-sheet-header-text-color": "#5f6368",
  "--palette-sheet-header-background": "#f8f9fa",
  "--palette-sheet-header-corner-background": "#f0f0f0",
  "--palette-sheet-header-selected-background": "#e8f0fe",
  "--palette-sheet-header-border-color": "#c0c0c0",
  "--palette-sheet-outline-color": "#1a73e8",
  "--palette-sheet-header-font":
    'bold 12px "Inter", "Adjusted Arial Fallback", sans-serif',
  "--palette-sheet-grid-separator-color": "#a0a0a0",
  "--palette-sheet-default-text-color": "#333333",
  "--palette-sheet-header-selected-color": "#1b1b1f",
};

// worksheetCanvas.ts reads `window.devicePixelRatio` at module load time, so
// this must run before that module is imported (the harness imports it
// dynamically afterwards).
export function installDomGlobals(devicePixelRatio: number): void {
  Object.assign(globalThis, {
    window: { devicePixelRatio },
    document: {
      createElement: (tagName: string) => new FakeElement(tagName),
      addEventListener: () => {},
      removeEventListener: () => {},
    },
    getComputedStyle: () => ({
      getPropertyValue: (name: string) => testThemeVars[name] ?? "",
    }),
  });
}
