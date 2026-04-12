#  IronCalc

## 📚 About

Xlsx importer and exporter for the IronCalc engine.

## Testing

This module test Excel compatibility. Dropping a file in `tests/calc_tests` will get automatically tested by the system.

Before any `cargo test` run `build.rs` builds a test for every file.

We test documentation files (found as examples in <https://docs.ironcalc.com>), templates and general test files

## Example files

You can run any of the example files by `cargo run --example file-name.rs`

## 🚴 Usage

The command

```
cargo build --release
```

Will produce a binary:

- `/target/release/test` you can use to test that IronCalc computes the same results as Excel on a particular file
