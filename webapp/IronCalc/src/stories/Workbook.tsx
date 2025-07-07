import { useEffect, useState } from "react";

import { IronCalc, Model, init } from "../index";

/** Primary UI component for user interaction */
export const Workbook = () => {
  const [model, setModel] = useState<Model | null>(null);

  useEffect(() => {
    async function start() {
      await init();
      const response = await fetch("example.ic");
      if (!response.ok) {
        setModel(new Model("Workbook1", "en", "UTC"));
      } else {
        const arrayBuffer = await response.arrayBuffer();
        const uint8Array = new Uint8Array(arrayBuffer);
        setModel(Model.from_bytes(uint8Array));
      }
    }
    start();
  }, []);
  if (!model) {
    return <div>Loading...</div>;
  }
  return (
    <div
      style={{
        position: "absolute",
        top: "0px",
        bottom: "0px",
        left: "0px",
        right: "0px",
      }}
    >
      <IronCalc model={model} />
    </div>
  );
};
