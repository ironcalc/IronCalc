import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import pkg from './package.json' with { type: 'json' };
import workbookPkg from '../IronCalc/package.json' with { type: 'json' };

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    fs: {
      // Allow serving files from one level up to the project root
      allow: ['../../../'],
    },
  },
  resolve: {
    dedupe: Object.keys(workbookPkg.peerDependencies ?? {}).filter(dep => dep in pkg.dependencies)
  }
})
