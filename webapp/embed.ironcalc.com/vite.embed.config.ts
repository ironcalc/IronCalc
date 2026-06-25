import { defineConfig } from "vite";

export default defineConfig({
  build: {
    lib: {
      entry: "./src/embed.ts",
      name: "IronCalcEmbed",
      formats: ["iife"],
      fileName: () => "embed.js",
    },
    outDir: "dist",
    emptyOutDir: false,
    minify: true,
  },
});