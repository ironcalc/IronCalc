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
      light: "/ironcalc-logo.svg",
      dark: "/ironcalc-logo-dark.svg",
      alt: "IronCalc Logo",
    },
    siteTitle: false,

    search: {
      provider: "local",
    },

    sidebar: [
      { text: "About IronCalc", link: "/index.md" },
      { text: "Blog", link: "https://blog.ironcalc.com/" },
      { text: "App", link: "https://app.ironcalc.com/" },
      {
        text: "Features",
        collapsed: false,
        items: [
          { text: "Formatting Values", link: "/features/formatting-values" },
          { text: "Using Styles", link: "/features/using-styles" },
          { text: "Keyboard Shortcuts", link: "/features/keyboard-shortcuts" },
        ],
      },
      {
        text: "Functions",
        collapsed: false,
        items: [
          { text: "Database", link: "/database" },
          { text: "Date and Time", link: "/date-and-time" },
          { text: "Engineering", link: "/engineering" },
          {
            text: "Financial",
            collapsed: true,
            link: "functions/financial",
            items: [
              { text: "EFFECT" },
              { text: "FV", link: "functions/financial/FV" },
              { text: "FVSCHEDULE" },
            ],
          },
          { text: "Information", link: "/information" },
          { text: "Logical", link: "/logical" },
          { text: "Lookup and Reference", link: "/lookup-and-reference" },
          { text: "Math and Trigonometry", link: "/math-and-trigonometry" },
          { text: "Statistical", link: "/statistical" },
          { text: "Text", link: "/text" },
        ],
      },
      {
        text: "Python bindings",
        collapsed: true,
        items: [{ text: "Practical Guide", link: "/python-bindings" }],
      },
      {
        text: "More",
        collapsed: true,
        items: [
          { text: "Unsupported Features", link: "/unsupported-features" },
          { text: "How to contribute", link: "/how-to-contribute" },
          { text: "Examples", link: "/examples" },
        ],
      },
    ],

    editLink: {
      pattern: "https://github.com/vuejs/vitepress/edit/main/docs/:path",
      text: "Edit this page on GitHub",
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
