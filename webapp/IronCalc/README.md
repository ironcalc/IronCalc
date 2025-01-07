# IronCalc Web App

The webapp is build with React and TypeScript. We use icons from [Lucide](https://lucide.dev/)

## Build

First thing build the wasm is the `../bindings/wasm` folder.

If you have nodejs installed you just need to:

```bash
npm install
```

## Local development


```bash
npm run dev
```

## Linter and formatting

We use [biome](https://biomejs.dev/):

```
npm run check
```

Will check for linter and formatting issues.

## Testing

We use vitest. Simply:

```
npm run test
```

Warning: There is only the testing infrastructure in place.

## Deploy

Deploying is a bit of a manual hassle right now:
To build a deployable frontend:

```bash
npm run build
```

Please copy the `inroncalc.svg` icon and the models you want to have as 'examples' in the internal 'ic' format.
I normally compress the wasm and js files with brotli

```
brotli wasm_bg-*****.wasm
```

Copy to the final destination and you are good to go.