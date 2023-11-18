lint:
	cargo fmt -- --check
	cargo clippy --all-targets --all-features

format:
	cargo fmt

tests: lint
	cargo test

clean:
	cargo clean
	rm -r -f base/target
	rm -r -f xlsx/target
	rm cargo-test-*
	rm base/cargo-test-*
	rm xlsx/cargo-test-*


coverage:
	CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test
	grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/html

docs:
	cargo doc --no-deps

.PHONY: lint format tests docs coverage all