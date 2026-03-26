import { resolve } from "node:path";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import svgr from "vite-plugin-svgr";
import pkg from "./package.json";

const EXTERNALS = ["@ironcalc/wasm", ...Object.keys(pkg.peerDependencies)];

function isExternal(id: string): boolean {
  return EXTERNALS.some(
    (externalPkg) => id === externalPkg || id.startsWith(`${externalPkg}/`),
  );
}

// https://vitejs.dev/config/
export default defineConfig({
  build: {
    lib: {
      entry: resolve(__dirname, "src/index.ts"),
      name: "IronCalc",
      // the proper extensions will be added
      fileName: "ironcalc",
      formats: ["es"],
    },
    rolldownOptions: {
      external: isExternal,
    },
  },
  plugins: [react(), svgr()],
  server: {
    fs: {
      // Allow serving files from one level up to the project root
      allow: [".."],
    },
  },
});
