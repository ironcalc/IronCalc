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

We will build different _skins_: in the terminal, as a desktop application or use it in your own web application.

# Docker

If you have docker installed just run:

```bash
docker compose up --build
```

head over to <http://localhost:2080> to test the application.

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
ironcalc = { git = "https://github.com/ironcalc/IronCalc", version = "0.5"}
```

And then use this code in `main.rs`:

```rust
use ironcalc::{
    base::{expressions::utils::number_to_column, Model},
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

See https://github.com/ironcalc

# Early testing

An early preview of the technology running entirely in your browser:

https://app.ironcalc.com


# Collaborators needed!. Call to action

We don't have a vibrant community just yet. This is the very stages of the project. But if you are passionate about code with high standards and no compromises, if you are looking for a project with high impact, if you are interested in a better, more open infrastructure for spreadsheets, whether you are a developer (rust, python, TypeScript, electron/tauri/anything else native app, React, you name it), a designer, an Excel power user who wants features, a business looking to integrate a MIT/Apache licensed spreadsheet in your own SaaS application join us!

The best place to start will be to join or [discord channel](https://discord.gg/zZYWfh3RHJ) or send us an email at hello@ironcalc.com.

Many have said it better before me:

> Folks wanted for hazardous journey. Low wages, bitter cold, long hours of complete darkness. Safe return doubtful. Honour and recognition in event of success.


# License

Licensed under either of

* [MIT license](LICENSE-MIT)
* [Apache license, version 2.0](LICENSE-Apache-2.0)

at your option.
