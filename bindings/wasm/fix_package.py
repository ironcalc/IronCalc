# wasm-pack does not add the `snippets/` directory (generated from
# `#[wasm_bindgen(inline_js = ...)]` blocks) to the `files` field of the
# package.json it emits, so `npm publish` would ship a wasm.js importing
# modules that are not in the tarball. See https://github.com/ironcalc/IronCalc/issues/1316
# Upstream wasm-pack bug: https://github.com/wasm-bindgen/wasm-pack/issues/1206
# This script can be removed once wasm-pack includes snippets in `files` itself.
import json

package_file = "pkg/package.json"

with open(package_file) as f:
    package = json.load(f)

if "snippets" not in package["files"]:
    package["files"].append("snippets")

# The XLSX helpers are built as a second wasm bundle and copied into pkg/ after
# wasm-pack runs, so wasm-pack never lists them. Ship them and expose the core
# engine at the package root and the helpers at `/xlsx`, so consumers can
# `import ... from "@ironcalc/wasm/xlsx"`.
for xlsx_file in ("xlsx.js", "xlsx.d.ts", "xlsx_bg.wasm"):
    if xlsx_file not in package["files"]:
        package["files"].append(xlsx_file)

package["exports"] = {
    ".": {"types": "./wasm.d.ts", "default": "./wasm.js"},
    "./xlsx": {"types": "./xlsx.d.ts", "default": "./xlsx.js"},
}

with open(package_file, "w") as f:
    json.dump(package, f, indent=2)
    f.write("\n")
