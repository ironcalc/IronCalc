import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import solidPlugin from "vite-plugin-solid";
import solidSvg from "vite-plugin-solid-svg";

export default defineConfig({
  plugins: [solid(), solidPlugin(), solidSvg()],
});
