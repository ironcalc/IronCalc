# IronCalc application

This is the front end deployed at https:://app.ironcalc.com

The xlsx import and export run in the browser through `@ironcalc/wasm-xlsx`,
which is built from `bindings/wasm` (it replaces the engine-only
`@ironcalc/wasm` that `@ironcalc/workbook` is built against, see
`vite.config.ts`). Before installing, build it and the workbook component:

```
cd bindings/wasm && make xlsx
cd webapp/IronCalc && npm install && npm run build
```

To build for production:

```
npm install
npm run build
```

A development build:

```
npm run dev
```
