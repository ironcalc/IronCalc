import initWasm, { Model } from "@ironcalc/wasm";
import IronCalc from "./IronCalc";
import i18n from "./i18n";
import { IronCalcIcon, IronCalcIconWhite, IronCalcLogo } from "./icons";

export type { IronCalcHandle } from "./IronCalc";
export type { ButtonProperties, ButtonVariant, ButtonSize } from "./components/Button/Button";
export { Button } from "./components/Button/Button";
export type { InputProperties, InputSize } from "./components/Input/Input";
export { Input } from "./components/Input/Input";
export type { AlertProperties } from "./components/Modal/Alert";
export { Alert } from "./components/Modal/Alert";
export type { ConfirmProperties } from "./components/Modal/Confirm";
export { Confirm } from "./components/Modal/Confirm";
export type { PromptProperties } from "./components/Modal/Prompt";
export { Prompt } from "./components/Modal/Prompt";
export { IronCalc, IronCalcIcon, IronCalcIconWhite, IronCalcLogo, Model };

export const init: typeof initWasm = async (module_or_path) => {
  const result = initWasm(module_or_path);
  await i18n.init();
  return await result;
};
