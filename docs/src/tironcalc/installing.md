---
layout: doc
outline: deep
lang: en-US
---

# Installing and running Tironcalc

## Build

```
cargo build --release
```

You will find the binary at `./target/release/tiron`.

## Documentation

Start empty project:

```
$ tiron
```

Load an existing Excel file:

```
$ tiron example.xlsx
```

- `Arrow Keys` to navigate cells
- `e` to edit a cell and enter the value or formula.
- `q` to quit and save
- `+` to add a sheet
- `s` to go to the next sheet
- `a` to go to the previous sheet
- `PgUp/PgDown` to navigate rows faster
