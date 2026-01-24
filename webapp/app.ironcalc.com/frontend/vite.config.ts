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
    dedupe: Object.keys(workbookPkg.peerDependencies).filter(dep => dep in pkg.dependencies)
  }
})
