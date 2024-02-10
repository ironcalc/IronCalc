# 📚 IronCalc

[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]
[![codecov](https://codecov.io/gh/ironcalc/IronCalc/graph/badge.svg?token=ASJX12CHNR)](https://codecov.io/gh/ironcalc/IronCalc)
[![docs-badge]][docs-url]

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/ironcalc/ironcalc/blob/master/LICENSE
[actions-badge]: https://github.com/ironcalc/ironcalc/actions/workflows/rust-build-test.yaml/badge.svg
[actions-url]: https://github.com/ironcalc/ironcalc/actions?query=workflow%3ACI+branch%3Amaster
[docs-url]: https://docs.rs/ironcalc
[docs-badge]: https://img.shields.io/docsrs/ironcalc?logo=rust&style=flat-square

IronCalc is a new, modern, work-in-progress spreadsheet engine and set of tools to work with spreadsheets in diverse settings.

This repository contains the main engine and the xlsx reader and writer.

Programmed in Rust, you will be able to use it from a variety of programming languages like Python, JavaScript (wasm), nodejs and possibly R, Julia or Go.

We will build different _skins_: in the terminal, as a desktop application or use it in you own web application.

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

Major milestones:

* MVP, version 0.5.0: We intend to have a working version by mid March 2024 (version 0.5, MVP)
* Stable, version 1.0.0 will come later in December 2024

MVP stands for _Minimum Viable Product_

## Version 0.5 or MVP (15 March 2024)

Version 0.5 includes the engine, javascript and nodejs bindings and a web application

Features of the engine include:

* Read and write xlsx files
* API to set and read values from cells
* Implemented 192 Excel functions
* Time functions with timezones
* Prepared for i18n but will only support English
* Wide test coverage

UI features of the web application (backed by the engine):

* Enter values and formulas. Browse mode
* Italics, bold, underline, horizontal alignment
* Number formatting
* Add/remove/rename sheets
* Copy/Paste extend values
* Keyboard navigation
* Delete/Add rows and columns
* Resize rows and columns
* Correct scrolling and navigation

## Version 1.0 or Stable (December 2024)

Minor milestones in the ROADMAD for version 1.0.0 (engine and UI):

* Implementation of arrays and array formulas
* Formula documentation and context help
* Merge cells
* Pivot tables
* Define name manager (mostly UI)
* Update main evaluation algorithm with a support graph
* Dynamic arrays (SORT, UNIQUE, ..)
* Full i18n support with different locales and languages
* Python bindings
* Full test coverage

I will be creating issues during the first two months of 2024

# Early testing

An early preview of the technology running entirely in your browser:

https://playground.ironcalc.com



# License

Licensed under either of

* [MIT license](LICENSE-MIT)
* [Apache license, version 2.0](LICENSE-Apache-2.0)

at your option.