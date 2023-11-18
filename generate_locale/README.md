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

Further information:

http://cldr.unicode.org/


## TODO:

* Add tests
* Checkout whole folder?

