# In some platforms, python is called python3
PYTHON := $(shell command -v python 2>/dev/null || command -v python3 2>/dev/null)

# If neither is found, fail immediately
ifeq ($(PYTHON),)
  $(error No python found. Please install python.)
endif

all:
	wasm-pack build --target web --scope ironcalc --release
	cp README.pkg.md pkg/README.md
	npx tsc types.ts --target esnext --module esnext
	$(PYTHON) fix_types.py
	rm -f types.js

tests:
	wasm-pack build --target nodejs && node tests/test.mjs

lint:
	cargo check
	cargo fmt -- --check
	cargo clippy --all-targets --all-features -- -D warnings

clean:
	cargo clean
	rm -rf pkg
	rm -f types.js

.PHONY: all lint clean tests
