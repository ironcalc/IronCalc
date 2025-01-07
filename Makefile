.PHONY: lint
lint:
	cargo fmt -- --check
	cargo clippy --all-targets --all-features -- -W clippy::unwrap_used -W clippy::expect_used -W clippy::panic -D warnings
	cd webapp/IronCalc/ && npm install && npm run check
	cd webapp/app.ironcalc.com/frontend/ && npm install && npm run check

.PHONY: format
format:
	cargo fmt

.PHONY: tests
tests: lint
	cargo test
	make remove-artifacts
	# Regretabbly we need to build the wasm twice, once for the nodejs tests
	# and a second one for the vitest.
	cd bindings/wasm/ && wasm-pack build --target nodejs && node tests/test.mjs && make
	cd webapp/IronCalc/ && npm run test
	cd bindings/python && ./run_tests.sh && ./run_examples.sh

.PHONY: remove-artifacts
remove-artifacts:
	rm -f xlsx/hello-calc.xlsx
	rm -f xlsx/hello-styles.xlsx
	rm -f xlsx/widths-and-heights.xlsx

.PHONY: clean
clean: remove-artifacts
	cargo clean
	rm -r -f base/target
	rm -r -f xlsx/target
	rm -r -f bindings/python/target
	rm -r -f bindings/wasm/targets
	rm -f cargo-test-*
	rm -f base/cargo-test-*
	rm -f xlsx/cargo-test-*

.PHONY: coverage
coverage:
	CARGO_INCREMENTAL=0 RUSTFLAGS='-C instrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test
	grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/html

.PHONY: docs
docs:
	cargo doc --no-deps
