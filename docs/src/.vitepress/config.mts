import { defineConfig } from "vitepress";

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "IronCalc Documentation",
  description: "The documentation of IronCalc",
  head: [["link", { rel: "icon", href: "/favicon-32x32.png" }]],

  markdown: {
    container: {
      tipLabel: " ",
      warningLabel: " ",
      dangerLabel: " ",
      infoLabel: " ",
      detailsLabel: "Details",
    },
    math: true,
  },

  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config

    logo: {
      light: "/ironcalc-docs-logo.svg",
      dark: "/ironcalc-docs-logo-dark.svg",
      alt: "IronCalc Logo",
    },
    siteTitle: false,

    search: {
      provider: "local",
    },

    nav: [
      { text: "Blog", link: "https://blog.ironcalc.com/" },
      { text: "App", link: "https://app.ironcalc.com/" },
    ],

    sidebar: [
      { text: "About IronCalc", link: "/index.md" },
      {
        text: "Features",
        collapsed: false,
        items: [
          { text: "Formatting Values", link: "/features/formatting-values" },
          { text: "Using Styles", link: "/features/using-styles" },
          { text: "Keyboard Shortcuts", link: "/features/keyboard-shortcuts" },
          { text: "Importing Files", link: "/features/importing-files" },
        ],
      },
      {
        text: "Functions",
        collapsed: false,
        items: [
          { text: "Database", link: "functions/database" },
          { text: "Date and Time", link: "functions/date-and-time" },
          { text: "Engineering", link: "functions/engineering" },
          {
            text: "Financial",
            collapsed: true,
            link: "functions/financial",
            items: [{ text: "FV", link: "functions/financial/FV" }],
          },
          { text: "Information", link: "functions/information" },
          { text: "Logical", link: "functions/logical" },
          {
            text: "Lookup and Reference",
            link: "functions/lookup-and-reference",
          },
          {
            text: "Math and Trigonometry",
            link: "functions/math-and-trigonometry",
          },
          { text: "Statistical", link: "functions/statistical" },
          { text: "Text", link: "functions/text" },
        ],
      },
      {
        text: "Python bindings",
        collapsed: true,
        items: [
          {
            text: "Practical Guide",
            link: "python-bindings/python-bindings-practical-guide",
          },
        ],
      },
      {
        text: "More",
        collapsed: true,
        items: [
          { text: "Unsupported Features", link: "more/unsupported-features" },
          { text: "How to contribute", link: "more/how-to-contribute" },
          {
            text: "Understanding Error Types",
            link: "more/understanding-error-types",
          },
        ],
      },
    ],

    editLink: {
      pattern: "https://github.com/ironcalc/ironcalc/edit/main/docs/:path",
      text: "Edit on GitHub",
    },

    lastUpdated: {
      text: "Updated at",
      formatOptions: {
        dateStyle: "full",
        timeStyle: "medium",
      },
    },

    socialLinks: [
      { icon: "github", link: "https://github.com/ironcalc" },
      { icon: "discord", link: "https://discord.gg/zZYWfh3RHJ" },
    ],
  },
});
