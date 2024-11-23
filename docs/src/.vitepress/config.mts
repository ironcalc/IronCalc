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
        text: "Web Application",
        collapsed: true,
        items: [
          { text: "About the web application", link: "/web-application/about" },
          { text: "Importing Files", link: "/web-application/importing-files" },
          { text: "Sharing Files", link: "/web-application/sharing-files" },
        ],
      },
      {
        text: "Features",
        collapsed: true,
        items: [
          { text: "Formatting Values", link: "/features/formatting-values" },
          { text: "Using Styles", link: "/features/using-styles" },
          { text: "Keyboard Shortcuts", link: "/features/keyboard-shortcuts" },
          {
            text: "Error Types",
            link: "/features/error-types",
          },
          { text: "Unsupported Features", link: "/features/unsupported-features" },
        ],
      },
      {
        text: "Functions",
        collapsed: true,
        items: [
          { text: "Database", link: "/functions/database" },
          { text: "Date and Time", link: "/functions/date-and-time" },
          { text: "Engineering", link: "/functions/engineering" },
          {
            text: "Financial",
            collapsed: true,
            link: "/functions/financial",
            items: [{ text: "FV", link: "/functions/financial/fv" }],
          },
          { text: "Information", link: "/functions/information" },
          { text: "Logical", link: "/functions/logical" },
          {
            text: "Lookup and Reference",
            link: "/functions/lookup-and-reference",
          },
          {
            text: "Math and Trigonometry",
            link: "/functions/math-and-trigonometry",
          },
          { text: "Statistical", link: "/functions/statistical" },
          { text: "Text", link: "/functions/text" },
        ],
      },
      {
        text: "Programming",
        collapsed: true,
        items: [
          {
            text: "Rust",
            link: "/programming/rust",
          },
          {
            text: "Python",
            link: "/programming/python-bindings",
          },
          {
            text: "JavScript",
            link: "/programming/javascript-bindings",
          },
        ],
      },
      {
        text: "TUI Application: Tironcalc",
        collapsed: true,
        items: [
          {
            text: "About Tironcalc",
            link: "/tironcalc/about",
          },
          {
            text: "Installing and basic usage",
            link: "/tironcalc/installing",
          },
        ],
      },
      {
        text: "Contributing",
        collapsed: true,
        items: [
          { text: "How to contribute", link: "/contributing/how-to-contribute" },
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
