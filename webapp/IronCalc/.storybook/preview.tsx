import { createTheme, ThemeProvider } from "@mui/material/styles";
import type { Preview } from "@storybook/react";
import { useEffect, useState } from "react";
import { I18nextProvider } from "react-i18next";
import i18n from "../src/i18n";
import { defaultTheme } from "../src/theme";

const crazyTheme = createTheme(defaultTheme, {
  palette: {
    common: {
      black: "#2f1616",
      white: "#ecc9c9",
    },
    primary: {
      main: "#F2994A",
      light: "#EFAA6D",
      dark: "#D68742",
      contrastText: "#dccece",
    },
    secondary: {
      main: "#2F80ED",
      light: "#4E92EC",
      dark: "#2B6EC8",
      contrastText: "#272525",
    },
    error: {
      main: "#EB5757",
      light: "#E77A7A",
      dark: "#CB4C4C",
      contrastText: "#272525",
    },
    warning: {
      main: "#F2C94C",
      light: "#EED384",
      dark: "#D6B244",
      contrastText: "#e3e9cf",
    },
  },
});

const themes = {
  default: defaultTheme,
  crazy: crazyTheme,
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
  const theme = themes[themeName] ?? defaultTheme;

  const [isLoaded, setIsLoaded] = useState(false);

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
    <I18nextProvider i18n={i18n}>
      <ThemeProvider theme={theme}>{children}</ThemeProvider>
    </I18nextProvider>
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
