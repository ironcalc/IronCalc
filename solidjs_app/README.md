# Web IronCalc

## Widgets

Toolbar
NavigationBar
FormulaBar
ColorPicker
Number Formatter
Border Picker


## Stack

Vite
TypeScript
SolidJs
Lucide Icons
BiomeJs
Storybook
pnpm

## Recreate

Install nodejs
Activate pnpm
    corepack enable pnpm
Create app    
    pnpm create vite
    pnpm install
add biomejs
    pnpm add --save-dev --save-exact @biomejs/biome
    pnpm biome init
add solidjs
add storybook
    pnpm dlx storybook@latest init
add i18n
    pnpm add @solid-primitives/i18n
(https://github.com/jfgodoy/vite-plugin-solid-svg)
add vite-plugin-solid-svg
add script: "restore": "cp node_modules/@ironcalc/wasm/wasm_bg.wasm node_modules/.vite/deps/",



## Usage

```bash
$ pnpm install # or npm install or yarn install
```

## Available Scripts

In the project directory, you can run:

### `pnpm run dev`

Runs the app in the development mode.<br>
Open [http://localhost:5173](http://localhost:5173) to view it in the browser.

### `pnpm run build`

Builds the app for production to the `dist` folder.<br>
It correctly bundles Solid in production mode and optimizes the build for the best performance.

The build is minified and the filenames include the hashes.<br>
Your app is ready to be deployed!

## Deployment

Learn more about deploying your application with the [documentations](https://vitejs.dev/guide/static-deploy.html)
