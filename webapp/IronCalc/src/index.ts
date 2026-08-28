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
export type { CheckboxProperties } from "./components/Checkbox/Checkbox";
export { Checkbox } from "./components/Checkbox/Checkbox";
export type { InputProperties, InputSize } from "./components/Input/Input";
export { Input } from "./components/Input/Input";
export type { MenuProperties } from "./components/Menu/Menu";
export { Menu } from "./components/Menu/Menu";
export { MenuDivider } from "./components/Menu/MenuDivider";
export type {
  MenuItemProperties,
  MenuItemWithSubmenuProps,
} from "./components/Menu/MenuItem";
export { MenuItem, MenuItemWithSubmenu } from "./components/Menu/MenuItem";
export type { ConfirmProperties } from "./components/Modal/Confirm";
export { Confirm } from "./components/Modal/Confirm";
export type { SwitchProperties } from "./components/Switch/Switch";
export { Switch } from "./components/Switch/Switch";
export type {
  ToggleButtonOption,
  ToggleButtonProperties,
} from "./components/ToggleButton/ToggleButton";
export { ToggleButton } from "./components/ToggleButton/ToggleButton";
export type { TooltipProperties } from "./components/Tooltip/Tooltip";
export { Tooltip } from "./components/Tooltip/Tooltip";
export type { IronCalcHandle } from "./IronCalc";
export { darkThemeVariables } from "./theme";
export { IronCalc, IronCalcIcon, IronCalcIconWhite, IronCalcLogo, Model };

export const init: typeof initWasm = async (module_or_path) => {
  const result = initWasm(module_or_path);
  await i18n.init();
  return await result;
};
