# IronCalc python bindings

With IronCalc you can create, read, evaluate and manipulate xlsx files:
cell values and formulas, styles and named styles, conditional formatting,
sheets, rows and columns, defined names and more.

## Installation

```bash
pip install ironcalc
```

## Quick start

There are two APIs:

* The **user API** (`UserModel`): the same high level API used by the IronCalc
  web application. Every action evaluates the workbook, keeps undo/redo
  history and produces diffs for collaboration.
* The **raw API** (`Model`): a low level API. Nothing is evaluated
  automatically (call `evaluate()` yourself), there is no undo/redo and no
  diffs. Faster and more flexible, but easier to get wrong.

```python
import ironcalc as ic

model = ic.UserModel("my-workbook")
model.set_user_input(0, 1, 1, "150")
model.set_user_input(0, 2, 1, "=A1*2")
print(model.get_formatted_cell_value(0, 2, 1))  # "300"

# style a range: bold with a yellow background
model.update_range_style(0, 1, 1, 2, 1, "font.b", "true")
model.update_range_style(0, 1, 1, 2, 1, "fill.color", "#FFFF00")

model.save_to_xlsx("my-workbook.xlsx")
```

More examples (styled reports, conditional formatting, CSV import,
collaboration diffs) live in `docs/examples`.

## Compile and test

```bash
./run_tests.sh      # build with maturin and run the pytest suite
./run_examples.sh   # run all the examples in docs/examples
```

Or manually:

```bash
python3 -m venv venv
source venv/bin/activate
pip install maturin pytest
maturin develop
pytest tests/
```

## Creating documentation

We use sphinx:

```bash
python -m venv venv
source venv/bin/activate
pip install maturin sphinx
maturin develop
sphinx-build -M html docs html
python -m http.server --directory html/html/
```
