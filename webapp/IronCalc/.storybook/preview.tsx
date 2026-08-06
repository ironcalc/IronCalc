import type { Preview } from "@storybook/react";
import { useEffect, useRef, useState } from "react";
import { I18nextProvider } from "react-i18next";
import i18n from "../src/i18n";
import {
  darkThemeVariables,
  defaultThemeVariables,
  setThemeVariables,
} from "../src/theme/theme";
import "../src/theme/theme.css";
import "../src/index.css";

const themes = {
  default: defaultThemeVariables,
  dark: darkThemeVariables,
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
      <I18nextProvider i18n={i18n}>{children}</I18nextProvider>
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
          { value: "dark", title: "Dark" },
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
