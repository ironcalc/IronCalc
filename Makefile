lint:
	cargo fmt -- --check
	cargo clippy --all-targets --all-features

format:
	cargo fmt

tests: lint
	cargo test
	./target/debug/documentation
	cmp functions.md wiki/functions.md || exit 1
	make remove-artifacts
	cd bindings/wasm/ && wasm-pack build --target nodejs && node tests/test.mjs

remove-artifacts:
	rm -f xlsx/hello-calc.xlsx
	rm -f xlsx/hello-styles.xlsx
	rm -f xlsx/widths-and-heights.xlsx
	rm -f functions.md

clean: remove-artifacts
	cargo clean
	rm -r -f base/target
	rm -r -f xlsx/target
	rm -f cargo-test-*
	rm -f base/cargo-test-*
	rm -f xlsx/cargo-test-*


coverage:
	CARGO_INCREMENTAL=0 RUSTFLAGS='-C instrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test
	grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/html

update-docs:
	cargo build
	./target/debug/documentation -o wiki/functions.md

docs:
	cargo doc --no-deps

.PHONY: lint format tests docs coverage all