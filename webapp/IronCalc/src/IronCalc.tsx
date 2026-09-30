import type { Model } from "@ironcalc/wasm";
import { forwardRef, useEffect, useImperativeHandle, useMemo } from "react";
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

interface IronCalcProperties {
  model: Model;
  themeVariables?: PartialIronCalcThemeVariables;
  rootContainer?: HTMLElement | null;
  /** When false, renders without the toolbar, the formula bar is read-only and all edits are blocked. */
  canEdit?: boolean;
  /** Bump to force a repaint when the model changed due to a remote update. */
  revision?: number;
}

export interface IronCalcHandle {
  setLanguage: (language: string) => void;
}

const IronCalc = forwardRef<IronCalcHandle, IronCalcProperties>(
  ({ themeVariables, model, rootContainer, canEdit = true, revision }, ref) => {
    const root = rootContainer ?? document.body;
    // Keep a single state object: it holds selection, scroll, etc.
    const workbookState = useMemo(() => new WorkbookState(), []);
    useEffect(() => {
      if (root.classList.contains("ic-root")) {
        console.warn("rootContainer already in use:", root);
      }
      root.classList.add("ic-root");
      return () => root.classList.remove("ic-root");
    }, [root]);

    useEffect(() => {
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
          <Workbook
            model={model}
            workbookState={workbookState}
            canEdit={canEdit}
            revision={revision}
          />
        </I18nextProvider>
      </div>
    );
  },
);

IronCalc.displayName = "IronCalc";

export default IronCalc;
