import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import svgr from 'vite-plugin-svgr';
import pkg from './package.json';
import workbookPkg from '../../IronCalc/package.json';

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), svgr()],
  server: {
    fs: {
      // Allow serving files from one level up to the project root
      allow: ['../../../'],
    },
  },
  resolve: {
    // @ironcalc/workbook imports the engine-only @ironcalc/wasm (an external
    // of its build). This app imports and exports xlsx files in the browser,
    // so it swaps in @ironcalc/wasm-xlsx, which has the same API plus
    // `Model.fromXlsx` and `Model.toXlsx`. Only one wasm binary is loaded.
    // The matching type-level swap is `paths` in tsconfig.app.json.
    alias: [
      {
        find: /^@ironcalc\/wasm$/,
        replacement: fileURLToPath(
          new URL('./node_modules/@ironcalc/wasm-xlsx/wasm.js', import.meta.url),
        ),
      },
    ],
    dedupe: Object.keys(workbookPkg.peerDependencies).filter(dep => dep in pkg.dependencies)
  }
})
