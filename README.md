# IronCalc

[![MIT licensed][mit-badge]][mit-url]
[![Apache 2.0 licensed][apache-badge]][apache-url]
[![Build Status][actions-badge]][actions-url]
[![Code coverage][codecov-badge]][codecov-url]
[![docs-badge]][docs-url]
[![Discord chat][discord-badge]][discord-url]

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/ironcalc/IronCalc/blob/main/LICENSE-MIT

[apache-badge]: https://img.shields.io/badge/License-Apache_2.0-blue.svg
[apache-url]: https://github.com/ironcalc/IronCalc/blob/main/LICENSE-Apache-2.0

[codecov-badge]: https://codecov.io/gh/ironcalc/IronCalc/graph/badge.svg?token=ASJX12CHNR
[codecov-url]: https://codecov.io/gh/ironcalc/IronCalc

[actions-badge]: https://github.com/ironcalc/ironcalc/actions/workflows/rust-build-test.yaml/badge.svg
[actions-url]: https://github.com/ironcalc/IronCalc/actions/workflows/rust-build-test.yaml?query=workflow%3ARust+branch%3Amain

[docs-url]: https://docs.rs/ironcalc
[docs-badge]: https://img.shields.io/docsrs/ironcalc?logo=rust&style=flat-square

[discord-badge]: https://img.shields.io/discord/1206947691058171904.svg?logo=discord&style=flat-square
[discord-url]: https://discord.gg/zZYWfh3RHJ

IronCalc is a new, modern, work-in-progress spreadsheet engine and set of tools to work with spreadsheets in diverse settings.

This repository contains the main engine and the xlsx reader and writer.

Programmed in Rust, you will be able to use it from a variety of programming languages like Python, JavaScript (wasm), nodejs and possibly R, Julia or Go.

We will build different _skins_: in the terminal, as a desktop application or use it in you own web application.

# Building

```bash
cargo build --release
```

# Testing, linting and code coverage

Test are run automatically and test coverage can always be found in [codecov](https://codecov.io/gh/ironcalc/IronCalc)

If you want to run the tests yourself:

```bash
make tests
```

Note that this runs unit tests, integration tests, linter tests and formatting tests.

If you want to run the code coverage yourself:
```bash
make coverage
cd target/coverage/html/
python -m http.server
```

# API Documentation

Documentation is published at: https://docs.rs/ironcalc/latest/ironcalc/

It might be generated locally

```bash
$ make docs
$ cd target/doc
$ python -m http.server
```

And visit <http://0.0.0.0:8000/ironcalc/>

# Simple example

Add the dependency to `Cargo.toml`:
```toml
[dependencies]
ironcalc = { git = "https://github.com/ironcalc/IronCalc", version = "0.1"}
```

And then use this code in `main.rs`:

```rust
use ironcalc::{
    base::{expressions::utils::number_to_column, model::Model},
    export::save_to_xlsx,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut model = Model::new_empty("hello-calc.xlsx", "en", "UTC")?;
    // Adds a square of numbers in the first sheet
    for row in 1..100 {
        for column in 1..100 {
            let value = row * column;
            model.set_user_input(0, row, column, format!("{}", value));
        }
    }
    // Adds a new sheet
    model.add_sheet("Calculation")?;
    // column 100 is CV
    let last_column = number_to_column(100).unwrap();
    let formula = format!("=SUM(Sheet1!A1:{}100)", last_column);
    model.set_user_input(1, 1, 1, formula);

    // evaluates
    model.evaluate();

    // saves to disk
    save_to_xlsx(&model, "hello-calc.xlsx")?;
    Ok(())
}
```

See more examples in the `examples` folder of the xlsx crate.

# ROADMAP

> [!WARNING]  
> This is work-in-progress. IronCalc in developed in the open. Expect things to be broken and change quickly until version 0.5

Major milestones:

* MVP, version 0.5.0: We intend to have a working version by mid March 2024 (version 0.5, MVP)
* Stable, version 1.0.0 will come later in December 2024

MVP stands for _Minimum Viable Product_

## Version 0.5 or MVP (early 2024)

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

# Early testing

An early preview of the technology running entirely in your browser:

https://playground.ironcalc.com


# Collaborators needed!. Call to action

We don't have a vibrant community just yet. This is the very stages of the project. But if you are passionate about code with high standards and no compromises, if you are looking for a project with high impact, if you are interested in a better, more open infrastructure for spreadsheets, whether you are a developer (rust, python, TypeScript, electron/tauri/anything else native app, React, you name it), a designer (we need a logo desperately!), an Excel power user who wants features, a business looking to integrate a MIT/Apache licensed spreadsheet in your own SaaS application join us!

The best place to start will be to join or [discord channel](https://discord.gg/zZYWfh3RHJ) or send us an email at hello@ironcalc.com.

Many have said it better before me:

> Folks wanted for hazardous journey. Low wages, bitter cold, long hours of complete darkness. Safe return doubtful. Honour and recognition in event of success.


# License

Licensed under either of

* [MIT license](LICENSE-MIT)
* [Apache license, version 2.0](LICENSE-Apache-2.0)

at your option.