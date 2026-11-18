import type { ComponentProps, FunctionComponent } from "react";
import BorderBottomIconSvg from "./border-bottom.svg?react";
import BorderCenterHIconSvg from "./border-center-h.svg?react";
import BorderCenterVIconSvg from "./border-center-v.svg?react";
import BorderInnerIconSvg from "./border-inner.svg?react";
import BorderLeftIconSvg from "./border-left.svg?react";
import BorderNoneIconSvg from "./border-none.svg?react";
import BorderOuterIconSvg from "./border-outer.svg?react";
import BorderRightIconSvg from "./border-right.svg?react";
import BorderTopIconSvg from "./border-top.svg?react";
import FxSvg from "./fx.svg?react";
import IronCalcIconSvg from "./ironcalc_icon.svg?react";
import IronCalcIconWhiteSvg from "./ironcalc_icon_white.svg?react";
import IronCalcLogoSvg from "./orange+black.svg?react";

// Explicit type for the svgr components (same as `vite-plugin-svgr/client`),
// so the published declarations don't reference `.svg?react` modules, which
// only exist inside this repo's build (#1759).
type IconComponent = FunctionComponent<
  ComponentProps<"svg"> & {
    title?: string;
    titleId?: string;
    desc?: string;
    descId?: string;
  }
>;

export const BorderBottomIcon: IconComponent = BorderBottomIconSvg;
export const BorderCenterHIcon: IconComponent = BorderCenterHIconSvg;
export const BorderCenterVIcon: IconComponent = BorderCenterVIconSvg;
export const BorderInnerIcon: IconComponent = BorderInnerIconSvg;
export const BorderLeftIcon: IconComponent = BorderLeftIconSvg;
export const BorderNoneIcon: IconComponent = BorderNoneIconSvg;
export const BorderOuterIcon: IconComponent = BorderOuterIconSvg;
export const BorderRightIcon: IconComponent = BorderRightIconSvg;
export const BorderTopIcon: IconComponent = BorderTopIconSvg;
export const Fx: IconComponent = FxSvg;
export const IronCalcIcon: IconComponent = IronCalcIconSvg;
export const IronCalcIconWhite: IconComponent = IronCalcIconWhiteSvg;
export const IronCalcLogo: IconComponent = IronCalcLogoSvg;
