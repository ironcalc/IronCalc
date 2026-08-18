# Two wasm-pack builds from this crate:
#   pkg/       — @ironcalc/wasm       (engine only, default)
#   pkg-xlsx/  — @ironcalc/wasm-xlsx  (engine + xlsx import/export)
#
# The xlsx APIs are gated on `--features xlsx`, so the default bundle stays
# the same size. Both builds go through wasm-pack (and therefore wasm-opt).

# In some platforms, python is called python3
PYTHON := $(shell command -v python 2>/dev/null || command -v python3 2>/dev/null)

# If neither is found, fail immediately
ifeq ($(PYTHON),)
  $(error No python found. Please install python.)
endif

all: deps
	wasm-pack build --target web --scope ironcalc --release
	wasm-pack build --target web --scope ironcalc --release --out-dir pkg-xlsx -- --features xlsx
	cp README.pkg.md pkg/README.md
	cp README.pkg-xlsx.md pkg-xlsx/README.md
	pnpm exec tsc types.ts --target esnext --module esnext
	$(PYTHON) fix_types.py pkg
	$(PYTHON) fix_types.py pkg-xlsx
	$(PYTHON) fix_package.py pkg
	$(PYTHON) fix_package.py pkg-xlsx @ironcalc/wasm-xlsx
	rm -f types.js
	node --test tests/test_package.mjs
	PKG_DIR=pkg-xlsx node --test tests/test_package.mjs

# Installs the pinned JS tooling (typescript, typedoc) from pnpm-lock.yaml
deps:
	pnpm install --frozen-lockfile

tests:
	wasm-pack build --target nodejs && node tests/test.mjs && node tests/test_cf.mjs
	wasm-pack build --target nodejs --out-dir pkg-xlsx -- --features xlsx && node tests/test_xlsx.mjs

lint:
	cargo check
	cargo fmt -- --check
	cargo clippy --all-targets --all-features -- -D warnings

clean:
	cargo clean
	rm -rf pkg pkg-xlsx
	rm -f types.js

.PHONY: all deps lint clean tests
