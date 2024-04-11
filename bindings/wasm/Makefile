all:
	wasm-pack build --target web --scope ironcalc
	cp README.pkg.md pkg/README.md
	tsc types.ts --target esnext --module esnext
	python fix_types.py

lint:
	cargo check
	cargo fmt -- --check
	cargo clippy --all-targets --all-features -- -D warnings

clean:
	cargo clean
	rm -rf pkg
	rm -f types.js

.PHONY: all lint clean
