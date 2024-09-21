# IronCalc python bindings

With IronCalc you can create, read and manipulate xlsx files.
You can manage sheets, add, remove, delete rename.
You can add cell values, retrieve them and most importantly you can evaluate spreadsheets.

## Installation

```bash
pip install ironcalc
```




## Compile and test

To compile this and test it:

```bash
$ python3 -m venv venv
$ source venv/bin/activate
$ pip install maturin
$ maturin develop
$ cd examples
examples $ python example.py
```

From there if you use `python` you can `import ironcalc`. You can either create a new file, read it from a JSON string or import from Excel.

Hopefully the API is straightforward.