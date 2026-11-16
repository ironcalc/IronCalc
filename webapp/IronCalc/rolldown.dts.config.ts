// Bundles the type declarations into a single dist/index.d.ts, matching the
// single dist/ironcalc.js that Vite builds. Shipping one .d.ts per source
// file broke consumers: those files kept extensionless relative imports,
// which fail under moduleResolution node16/nodenext, plus `.svg?react` and
// `.css` imports that don't exist in dist (#1759).
import { defineConfig } from "rolldown";
import { dts } from "rolldown-plugin-dts";
import pkg from "./package.json" with { type: "json" };

const EXTERNALS = ["@ironcalc/wasm", ...Object.keys(pkg.peerDependencies)];

export default defineConfig({
  input: { index: "src/index.ts" },
  external: (id) =>
    EXTERNALS.some((name) => id === name || id.startsWith(`${name}/`)),
  plugins: [dts({ tsconfig: "tsconfig.json", emitDtsOnly: true })],
  output: { dir: "dist" },
});
