import type { StorybookConfig } from "@storybook/react-vite";

const config: StorybookConfig = {
  stories: ["../src/**/*.mdx", "../src/**/*.stories.@(js|jsx|mjs|ts|tsx)"],
  addons: [],
  framework: {
    name: "@storybook/react-vite",
    options: {},
  },
  viteFinal: (config) => {
    if (!config.server) {
      config.server = {};
    }
    if (!config.server.fs) {
      config.server.fs = {};
    }
    config.server.fs.allow = ["../.."];
    return config;
  }
};
export default config;
