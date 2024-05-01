# TironCalc

[![Discord chat][discord-badge]][discord-url]

[discord-badge]: https://img.shields.io/discord/1206947691058171904.svg?logo=discord&style=flat-square
[discord-url]: https://discord.gg/zZYWfh3RHJ

TironCalc, or Tiron for friends, is a TUI (Terminal User Interface) for IronCalc. Based on [ratatui](https://github.com/ratatui-org/ratatui)

![TironCalc Screenshot](screenshot.png)

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

-   `e` to edit a cell and enter the value or formula.
-   `q` to quit and save
-   `+` to add a sheet
-   `s` to go to the next sheet
-   `PgUp/PgDown` to navigate rows faster
-   `u` undo changes
-   `U` redo changes
-   `r` insert row
-   `c` insert column
-   `C` delete column
-   `R` delete row
-   `


## Inspiration

James Gosling of Java fame created [sc](https://en.wikipedia.org/wiki/Sc_(spreadsheet_calculator)) the spreadsheet calculator.

Andrés Martinelli has been maintaining [sc-im](https://github.com/andmarti1424/sc-im), the spreadsheet calculator improvised.
