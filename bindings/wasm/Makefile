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

# Content hash of everything that feeds the wasm builds. The build targets are
# stamp files named after it, so `make wasm` / `make xlsx` are no-ops while the
# sources are unchanged.
# Requires a git checkout.
SRC_HASH := $(shell cd ../.. && git ls-files -co --exclude-standard -- bindings/wasm base xlsx Cargo.toml Cargo.lock | sort | while IFS= read -r f; do [ -f "$$f" ] && printf '%s\n' "$$f"; done | git hash-object --stdin-paths | git hash-object --stdin)

all: wasm xlsx

wasm: pkg/.built-$(SRC_HASH)

# `deps` is order-only (after the |): being .PHONY it would otherwise force
# the stamp to rebuild on every run.
pkg/.built-$(SRC_HASH): | deps
	wasm-pack build --target web --scope ironcalc --release
	cp README.pkg.md pkg/README.md
	pnpm exec tsc types.ts --target esnext --module esnext
	$(PYTHON) fix_types.py pkg
	$(PYTHON) fix_package.py pkg
	rm -f types.js
	node --test tests/test_package.mjs
	rm -f pkg/.built-*
	touch $@

xlsx: pkg-xlsx/.built-$(SRC_HASH)

pkg-xlsx/.built-$(SRC_HASH): | deps
	wasm-pack build --target web --scope ironcalc --release --out-dir pkg-xlsx -- --features xlsx
	cp README.pkg-xlsx.md pkg-xlsx/README.md
	pnpm exec tsc types.ts --target esnext --module esnext
	$(PYTHON) fix_types.py pkg-xlsx
	$(PYTHON) fix_package.py pkg-xlsx @ironcalc/wasm-xlsx
	rm -f types.js
	PKG_DIR=pkg-xlsx node --test tests/test_package.mjs
	rm -f pkg-xlsx/.built-*
	touch $@

# Installs the pinned JS tooling (typescript, typedoc) from pnpm-lock.yaml
deps:
	pnpm install --frozen-lockfile

# The nodejs test builds go to their own out-dirs so they don't clobber the
# publishable web builds in pkg/ and pkg-xlsx/. They use --dev (no
# optimizations, no wasm-opt) since they only exist to run the tests.
tests:
	wasm-pack build --dev --target nodejs --out-dir pkg-nodejs && node tests/test.mjs && node tests/test_cf.mjs
	wasm-pack build --dev --target nodejs --out-dir pkg-nodejs-xlsx -- --features xlsx && node tests/test_xlsx.mjs

lint:
	cargo check
	cargo fmt -- --check
	cargo clippy --all-targets --all-features -- -D warnings

clean:
	cargo clean
	rm -rf pkg pkg-xlsx pkg-nodejs pkg-nodejs-xlsx
	rm -f types.js

.PHONY: all wasm xlsx deps lint clean tests
