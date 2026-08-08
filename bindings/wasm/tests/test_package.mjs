// Regression test for https://github.com/ironcalc/IronCalc/issues/1316
//
// wasm-pack emits `snippets/ironcalc_base-<hash>/inline0.js` (from the
// `#[wasm_bindgen(inline_js = ...)]` block in base/src/tz/browser_tz.rs) and
// wasm.js imports it, but it does not add `snippets` to the `files` field of
// pkg/package.json — so `npm publish` shipped a wasm.js whose imports could
// not be resolved. This test packs pkg/ exactly like `npm publish` does and
// asserts every module wasm.js imports is actually in the tarball.
import test from 'node:test';
import assert from 'node:assert';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const pkgDir = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'pkg');

test('npm tarball contains every module imported by wasm.js', () => {
    const wasmJs = readFileSync(path.join(pkgDir, 'wasm.js'), 'utf8');
    const imports = new Set();
    for (const match of wasmJs.matchAll(/from\s+["'](\.\/[^"']+)["']/g)) {
        imports.add(match[1].replace(/^\.\//, ''));
    }

    const [{ files }] = JSON.parse(
        execFileSync('npm', ['pack', '--dry-run', '--json'], { cwd: pkgDir })
    );
    const packed = new Set(files.map((file) => file.path));

    assert.ok(packed.has('wasm.js'), 'tarball must contain wasm.js');
    assert.ok(packed.has('wasm_bg.wasm'), 'tarball must contain wasm_bg.wasm');
    assert.ok(packed.has('wasm.d.ts'), 'tarball must contain wasm.d.ts');
    for (const specifier of imports) {
        assert.ok(
            packed.has(specifier),
            `wasm.js imports "./${specifier}" but it is not in the npm tarball ` +
                '(is "snippets" missing from "files" in pkg/package.json?)'
        );
    }
});
