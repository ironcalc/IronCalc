# Generate Locale

This is a small util to generate locales for IronCalc.

To build

```bash
$ cargo build --release
```

To run it you will need a checkout of the [CLDR json repo](https://github.com/unicode-org/cldr-json)

```bash
$ generate_locale --locales=<locales-file> --cldr-dir=<cldr-dir> --output=<output-file>
```

It writes the json output file and a `locales.bin` (the same data encoded
with bitcode) in the current directory. Copy `locales.bin` to
`base/src/locale/locales.bin` and the json to `generate_locale/locales.json`.

Further information:

http://cldr.unicode.org/

## Data version and license

The locale data is derived from Unicode CLDR and redistributed under the
Unicode License v3 (`Unicode-3.0`); the notice lives in
`base/THIRD-PARTY-NOTICES.md` and is shipped with every package that
contains `locales.bin`.

The data currently in the repository was generated from cldr-json
**48.2.0** (CLDR 48.2). The json output records the version it was
generated from in its `_meta` entry (read from
`cldr-json/cldr-core/package.json` in the checkout). When you regenerate
the data, update the version in `base/THIRD-PARTY-NOTICES.md` and here.

## TODO:

* Add tests
* Checkout whole folder?
