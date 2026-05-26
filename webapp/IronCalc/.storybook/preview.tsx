import type { Preview } from "@storybook/react";
import { useEffect, useRef, useState } from "react";
import { I18nextProvider } from "react-i18next";
import { PortalProvider } from "../src/components/PortalContext";
import i18n from "../src/i18n";
import type { PartialIronCalcThemeVariables } from "../src/theme";
import { defaultThemeVariables, setThemeVariables } from "../src/theme/theme";
import "../src/theme/theme.css";

const crazyThemeVariables: PartialIronCalcThemeVariables = {
  "--palette-common-black": "#2f1616",
  "--palette-common-white": "#ecc9c9",

  "--palette-primary-main": "#F2994A",
  "--palette-primary-light": "#EFAA6D",
  "--palette-primary-dark": "#D68742",
  "--palette-primary-contrast-text": "#dccece",

  "--palette-secondary-main": "#2F80ED",
  "--palette-secondary-light": "#4E92EC",
  "--palette-secondary-dark": "#2B6EC8",
  "--palette-secondary-contrast-text": "#272525",

  "--palette-error-main": "#EB5757",
  "--palette-error-light": "#E77A7A",
  "--palette-error-dark": "#CB4C4C",
  "--palette-error-contrast-text": "#272525",

  "--palette-warning-main": "#F2C94C",
  "--palette-warning-light": "#EED384",
  "--palette-warning-dark": "#D6B244",
  "--palette-warning-contrast-text": "#e3e9cf",
};

const themes = {
  default: defaultThemeVariables,
  crazy: crazyThemeVariables,
};

function PreviewProviders({
  children,
  themeName,
  locale,
}: {
  children: React.ReactNode;
  themeName: keyof typeof themes;
  locale: string;
}) {
  const [isLoaded, setIsLoaded] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const themeVariables = themes[themeName];
    if (rootRef.current && themeVariables) {
      setThemeVariables(themeVariables, rootRef.current);
    }
  }, [themeName]);

  useEffect(() => {
    async function start() {
      await i18n.init();
      if (i18n.language !== locale) {
        void i18n.changeLanguage(locale);
      }
      setIsLoaded(true);
    }
    start();
  }, [locale]);

  if (!isLoaded) {
    return null;
  }

  return (
    <div ref={rootRef} className="ic-root">
      <PortalProvider>
        <I18nextProvider i18n={i18n}>{children}</I18nextProvider>
      </PortalProvider>
    </div>
  );
}

const preview: Preview = {
  globalTypes: {
    theme: {
      name: "Theme",
      description: "Global theme for IronCalc",
      defaultValue: "default",
      toolbar: {
        icon: "paintbrush",
        items: [
          { value: "default", title: "Default" },
          { value: "crazy", title: "Crazy" },
        ],
      },
    },
    locale: {
      name: "Locale",
      description: "Global locale",
      defaultValue: "en-US",
      toolbar: {
        icon: "globe",
        items: [
          { value: "en-US", title: "English" },
          { value: "es-ES", title: "Español" },
          { value: "fr-FR", title: "Français" },
          { value: "de-DE", title: "Deutsch" },
          { value: "it-IT", title: "Italiano" },
        ],
      },
    },
  },
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
  },
  decorators: [
    (Story, context) => {
      const themeName =
        (context.globals.theme as keyof typeof themes) ?? "default";
      const locale = (context.globals.locale as string) ?? "en-US";

      return (
        <PreviewProviders themeName={themeName} locale={locale}>
          <Story />
        </PreviewProviders>
      );
    },
  ],
};

export default preview;
