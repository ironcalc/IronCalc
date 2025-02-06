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
npm run storybook
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

## Build package


```bash
npm run build
```
