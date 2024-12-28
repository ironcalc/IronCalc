all:
	wasm-pack build --target web --scope ironcalc --release
	cp README.pkg.md pkg/README.md
	tsc types.ts --target esnext --module esnext
	python fix_types.py
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

.PHONY: all lint clean
