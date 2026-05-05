import initWasm, { Model } from "@ironcalc/wasm";
import IronCalc from "./IronCalc";
import i18n from "./i18n";
import { IronCalcIcon, IronCalcIconWhite, IronCalcLogo } from "./icons";

export type {
  ButtonProperties,
  ButtonSize,
  ButtonVariant,
} from "./components/Button/Button";
export { Button } from "./components/Button/Button";
export type { IconButtonProperties } from "./components/Button/IconButton";
export { IconButton } from "./components/Button/IconButton";
export type { InputProperties, InputSize } from "./components/Input/Input";
export { Input } from "./components/Input/Input";
export { useModalFocus } from "./components/Modal/useModalFocus";
export type { TooltipProperties } from "./components/Tooltip/Tooltip";
export { Tooltip } from "./components/Tooltip/Tooltip";
export type { IronCalcHandle } from "./IronCalc";
export { IronCalc, IronCalcIcon, IronCalcIconWhite, IronCalcLogo, Model };

export const init: typeof initWasm = async (module_or_path) => {
  const result = initWasm(module_or_path);
  await i18n.init();
  return await result;
};
