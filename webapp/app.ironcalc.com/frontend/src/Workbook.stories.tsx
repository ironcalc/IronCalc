import { IronCalc, init, Model } from "@ironcalc/workbook";
import workbookCss from "@ironcalc/workbook/style.css?inline";
import type { Meta, StoryObj } from "@storybook/react";
import { useEffect, useState } from "react";
import { createPortal } from "react-dom";

// Obnoxious *inherited* styles on <body>, applied outside the shadow root.
// CSS inheritance crosses the shadow boundary via the host element, so any
// widget text that doesn't explicitly reset these on its own root picks them
// up — exactly what happens in a consumer app with its own body styles.
//
// `!important` stands in for high-specificity rules a real consumer stylesheet
// would have; a well-behaved library sets its own color/font-family on its
// root wrapper with equivalent specificity.
const leakWarning = document.createElement("style");
leakWarning.textContent = `
  body {
    color: #ff00aa !important;
    font-family: "Comic Sans MS", cursive, sans-serif !important;
    font-size: 22px !important;
    background: repeating-linear-gradient(
      45deg,
      #00ffaa 0 20px,
      #ffea00 20px 40px
    ) !important;
  }
`;

function ShadowWorkbook() {
  const [model, setModel] = useState<Model | null>(null);
  const [shadow, setShadow] = useState<ShadowRoot | null>(null);

  useEffect(() => {
    async function start() {
      await init();
      setModel(new Model("Workbook1", "en", "UTC", "en"));
    }
    start();
  }, []);

  useEffect(() => {
    document.head.appendChild(leakWarning);
    return () => {
      leakWarning.remove();
    };
  }, []);

  const attach = (host: HTMLDivElement | null) => {
    if (host && !host.shadowRoot) {
      const root = host.attachShadow({ mode: "open" });
      const style = document.createElement("style");
      style.textContent = workbookCss;
      root.appendChild(style);
      setShadow(root);
    }
  };

  if (!model) {
    return <div>Loading...</div>;
  }
  return (
    <div ref={attach} style={{ position: "absolute", inset: 0 }}>
      {shadow && createPortal(<IronCalc model={model} />, shadow)}
    </div>
  );
}

const meta = {
  title: "Consumer/Workbook in shadow root",
  component: ShadowWorkbook,
} satisfies Meta<typeof ShadowWorkbook>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {};
