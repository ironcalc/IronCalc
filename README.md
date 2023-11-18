# 📚 IronCalc

IronCalc is a new, modern, work-in-progress spreadsheet engine and set of tools to work with spreadsheets in diverse settings.

This repository contains the main engine and the xlsx importer and exporter.

Programmed in Rust, you can use it from a variety of programming languages like [Python](https://github.com/ironcalc/bindings-python), [JavaScript (wasm)](https://github.com/ironcalc/bindings-js), [nodejs](https://github.com/ironcalc/bindings-nodejs) and soon R, Julia, Go and possibly others.

It has several different _skins_. You can use it in the [terminal](https://github.com/ironcalc/skin-terminal), as a [desktop application](https://github.com/ironcalc/bindings-desktop) or use it in you own [web application](https://github.com/ironcalc/skin-web).

# 🛠️ Building

```bash
cargo build --release
```

# Testing, linting and code coverage

Testing:
```bash
cargo test
```

Linting:
```bash
make lint
```

Testing and linting:
```bash
make tests
```

Code coverage:
```bash
make coverage
cd target/coverage/html/
python -m http.server
```

# 🖹 API Documentation

Documentation might be generated with

```bash
$ cargo doc --no-deps
```

# 📝 ROADMAP

> [!WARNING]  
> This is work-in-progress. IronCalc in developed in the open. Expect things to be broken and change quickly until version 0.5

* We intend to have a working version by mid January 2024 (version 0.5, MVP)
* Version 1.0.0 will come later in 2024


# License

Licensed under either of

* [MIT license](LICENSE-MIT)
* [Apache license, version 2.0](LICENSE-Apache-2.0)

at your option.