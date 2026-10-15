# IronCalc Web bindings with XLSX import/export

This package is the IronCalc engine plus XLSX import and export. If you only need the calc engine, use [`@ironcalc/wasm`](https://www.npmjs.com/package/@ironcalc/wasm) instead.

## Usage

In your project

```
npm install @ironcalc/wasm-xlsx
```

And then in your TypeScript

```TypeScript
import init, { Model } from "@ironcalc/wasm-xlsx";

await init();

async function convert() {
    const xlsxBytes = new Uint8Array(await file.arrayBuffer());
    const model = Model.fromXlsx(xlsxBytes, "Workbook1", "en", "UTC", "en");

    model.setUserInput(0, 1, 1, "23");
    model.setUserInput(0, 1, 2, "=A1*3+1");

    const out = model.toXlsx();
}

convert();
```

### Large files

`Model.fromXlsx` needs the whole file on the JavaScript side, which is then
copied into wasm memory. For large files, stream the file into a `ByteBuffer`
instead, so only one chunk at a time lives outside wasm memory:

```TypeScript
import init, { ByteBuffer, Model } from "@ironcalc/wasm-xlsx";

await init();

async function convertLarge(file: File) {
    const CHUNK = 8 * 1024 * 1024;
    const buffer = new ByteBuffer(file.size);
    for (let offset = 0; offset < file.size; offset += CHUNK) {
        const slice = file.slice(offset, offset + CHUNK);
        buffer.append(new Uint8Array(await slice.arrayBuffer()));
    }
    // Consumes the buffer
    const model = Model.fromXlsxBuffer(buffer, "Workbook1", "en", "UTC", "en");
}
```

`Model.fromBuffer(buffer, "en")` does the same for a workbook in IronCalc's own
format (the bytes from `model.toBytes()`), and is also available in
`@ironcalc/wasm`.
