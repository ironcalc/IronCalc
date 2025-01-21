# CHANGELOG

## Unreleased

### Added

- New function UNICODE ([#128](https://github.com/ironcalc/IronCalc/pull/128))
- New document server (Thanks Dani!)
- New function FORMULATEXT
- Name Manager ([#212](https://github.com/ironcalc/IronCalc/pull/212) [#220](https://github.com/ironcalc/IronCalc/pull/220))
- Add context menu. We can now insert rows and columns. Freeze and unfreeze rows and columns. Delete rows and columns [#271]
- Add nodejs bindings [#254]
- Add python bindings for all platforms
- Add is split into the product and widget
- Add Python documentation [#260]

### Fixed

- Fixed several issues with pasting content
- Fixed several issues with borders
- Fixed bug where columns and rows could be resized to negative width and height, respectively
- Undo/redo when add/delete sheet now works [#270]
- Numerous small fixes
- Multiple fixes to the documentation

## [0.2.0] - 2024-11-06 (The HN release)

### Added

- Rust crate ironcalc_base
- Rust crate ironcalc
- Minimal Python bindings (only Linux)
- JavaScript bindings
- React WebApp

[0.2.0]: https://github.com/IronCalc/ironcalc/releases/tag/v0.2.0
