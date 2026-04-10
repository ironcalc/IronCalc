import initWasm, { Model } from "@ironcalc/wasm";
import IronCalc from "./IronCalc";
import i18n from "./i18n";
import { IronCalcIcon, IronCalcIconWhite, IronCalcLogo } from "./icons";

export type { IronCalcHandle } from "./IronCalc";
export { IronCalc, IronCalcIcon, IronCalcIconWhite, IronCalcLogo, Model };

export const init: typeof initWasm = async (module_or_path) => {
  const result = initWasm(module_or_path);
  await i18n.init();
  return await result;
};
