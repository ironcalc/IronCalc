# ---------------------------------------------------------------------------
# Build strategy
# ---------------------------------------------------------------------------
# We need two separate WASM bundles in the same NPM package:
#   1) Core engine  →  @ironcalc/wasm            (small, always shipped)
#   2) XLSX helpers →  @ironcalc/wasm/xlsx       (large, optional)
#
# The natural way would be two wasm-pack invocations – one default lib and one
#   wasm-pack build --bin xlsx_wasm …
# Unfortunately, on the stable Rust tool-chain wasm-pack's `--bin` flag relies
# on Cargo's unstable `--out-dir`/`--artifact-dir` feature and therefore fails.
#
# Until that becomes stable (or we switch to a nightly tool-chain), we compile
# the second binary with plain `cargo build` and then run `wasm-bindgen`
# ourselves.  As soon as the flag is stabilised we can collapse back to the
# two simple wasm-pack commands shown above and delete the hand-rolled steps.
# ---------------------------------------------------------------------------
# In some platforms, python is called python3
PYTHON := $(shell command -v python 2>/dev/null || command -v python3 2>/dev/null)

# If neither is found, fail immediately
ifeq ($(PYTHON),)
  $(error No python found. Please install python.)
endif

TEMP_DIR := pkg_xlsx

all: deps
	wasm-pack build --target web --scope ironcalc --release

	# Build XLSX helper separately (cargo + wasm-bindgen)
	cargo build --release --target wasm32-unknown-unknown --bin xlsx_wasm
	wasm-bindgen --target web --typescript --out-dir $(TEMP_DIR) --out-name xlsx ../../target/wasm32-unknown-unknown/release/xlsx_wasm.wasm

	# Move generated files into main pkg directory
	mv $(TEMP_DIR)/xlsx_bg.wasm pkg/xlsx_bg.wasm
	mv $(TEMP_DIR)/xlsx.js pkg/xlsx.js
	mv $(TEMP_DIR)/xlsx.d.ts pkg/xlsx.d.ts

	cp README.pkg.md pkg/README.md
	pnpm exec tsc types.ts --target esnext --module esnext
	$(PYTHON) fix_types.py
	$(PYTHON) fix_package.py
	rm -f types.js
	rm -rf $(TEMP_DIR)
	node --test tests/test_package.mjs

# Installs the pinned JS tooling (typescript, typedoc) from pnpm-lock.yaml
deps:
	pnpm install --frozen-lockfile

tests:
	wasm-pack build --target nodejs
	cargo build --release --target wasm32-unknown-unknown --bin xlsx_wasm
	wasm-bindgen --target nodejs --typescript --out-dir $(TEMP_DIR) --out-name xlsx ../../target/wasm32-unknown-unknown/release/xlsx_wasm.wasm
	mv $(TEMP_DIR)/xlsx_bg.wasm pkg/xlsx_bg.wasm || true
	mv $(TEMP_DIR)/xlsx.js pkg/xlsx.js || true
	mv $(TEMP_DIR)/xlsx.d.ts pkg/xlsx.d.ts || true
	rm -rf $(TEMP_DIR)
	node tests/test.mjs && node tests/test_cf.mjs

lint:
	cargo check
	cargo fmt -- --check
	cargo clippy --all-targets --all-features -- -D warnings

clean:
	cargo clean
	rm -rf pkg
	rm -f types.js

.PHONY: all deps lint clean tests
