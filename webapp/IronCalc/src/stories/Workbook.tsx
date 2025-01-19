import { useEffect, useState } from "react";

import { IronCalc, Model, init } from "../index";

// export interface IronCalcProps {}

/** Primary UI component for user interaction */
export const Workbook = () => {
  const [model, setModel] = useState<Model | null>(null);

  useEffect(() => {
    async function start() {
      await init();
      setModel(new Model("Workbook1", "en", "UTC"));
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
