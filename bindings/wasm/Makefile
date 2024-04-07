all:
	wasm-pack build --target web --scope ironcalc
	cp README.pkg.md pkg/README.md

lint:
	cargo check
	cargo fmt -- --check
	cargo clippy --all-targets --all-features -- -D warnings

clean:
	cargo clean
	rm -rf pkg

.PHONY: all web lint