// Kept separate from vite.config.ts so `tsc -b` does not depend on vitest
// being installed. Vitest picks this file up automatically.
import { defineConfig, mergeConfig } from "vitest/config";
import viteConfig from "./vite.config";

export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      environment: "jsdom",
      globals: false,
    },
  }),
);
