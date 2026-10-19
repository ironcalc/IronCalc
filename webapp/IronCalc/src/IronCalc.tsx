import type { Model } from "@ironcalc/wasm";
import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { I18nextProvider } from "react-i18next";
import Workbook from "./components/Workbook/Workbook.tsx";
import { WorkbookState } from "./components/workbookState.ts";
import i18n from "./i18n";
import "./theme/theme.css";
import "./index.css";
import {
  type PartialIronCalcThemeVariables,
  setThemeVariables,
  unsetThemeVariables,
} from "./theme";
import {
  type ColorScheme,
  ColorSchemeContext,
  InvertColorOptionsContext,
} from "./theme/colorScheme";
import { DEFAULT_OPTIONS, type InvertColorOptions } from "./theme/invertColor";

interface IronCalcProperties {
  model: Model;
  themeVariables?: PartialIronCalcThemeVariables;
  /**
   * "dark" shows every workbook color (fills, fonts, borders, conditional
   * formats, tab colors, theme accents...) inverted for a dark background,
   * while the workbook itself keeps its light-mode colors. Pair it with
   * `darkThemeVariables` for the chrome. Defaults to "light".
   */
  colorScheme?: ColorScheme;
  rootContainer?: HTMLElement | null;
  /** When false, renders without the toolbar, the formula bar is read-only and all edits are blocked. */
  canEdit?: boolean;
}

export interface IronCalcHandle {
  setLanguage: (language: string) => void;
}

const IronCalc = forwardRef<IronCalcHandle, IronCalcProperties>(
  (
    {
      themeVariables,
      colorScheme = "light",
      model,
      rootContainer,
      canEdit = true,
    },
    ref,
  ) => {
    const root = rootContainer ?? document.body;
    // One runtime state for the lifetime of the widget: IronCalc re-renders
    // when the inversion knobs change and must not reset the editing state
    const workbookState = useRef(new WorkbookState()).current;
    // Knobs of the dark-mode color inversion, tunable from the regional
    // settings drawer while in dark mode
    const [invertColorOptions, setInvertColorOptions] =
      useState<InvertColorOptions>(DEFAULT_OPTIONS);
    const invertColorState = useMemo(
      () => ({
        options: invertColorOptions,
        setOptions: setInvertColorOptions,
      }),
      [invertColorOptions],
    );
    useEffect(() => {
      if (root.classList.contains("ic-root")) {
        console.warn("rootContainer already in use:", root);
      }
      root.classList.add("ic-root");
      return () => root.classList.remove("ic-root");
    }, [root]);

    useLayoutEffect(() => {
      if (themeVariables) {
        setThemeVariables(themeVariables, root);
        return () => unsetThemeVariables(root);
      }
    }, [root, themeVariables]);

    useImperativeHandle(ref, () => ({
      setLanguage(language: string) {
        if (i18n.language !== language) {
          i18n.changeLanguage(language);
          const lang = language.split("-")[0];
          model.setLanguage(lang);
        }
      },
    }));

    return (
      <div className="ic-widget">
        <I18nextProvider i18n={i18n}>
          <ColorSchemeContext.Provider value={colorScheme}>
            <InvertColorOptionsContext.Provider value={invertColorState}>
              <Workbook
                model={model}
                workbookState={workbookState}
                canEdit={canEdit}
              />
            </InvertColorOptionsContext.Provider>
          </ColorSchemeContext.Provider>
        </I18nextProvider>
      </div>
    );
  },
);

IronCalc.displayName = "IronCalc";

export default IronCalc;
