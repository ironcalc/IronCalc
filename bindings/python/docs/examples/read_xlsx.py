"""Read an existing xlsx file, inspect it and recalculate it."""

import os
import tempfile

import ironcalc as ic

# First create a workbook to read (in real life you already have one)
folder = tempfile.mkdtemp()
file = os.path.join(folder, "input.xlsx")

writer = ic.UserModel("input")
writer.set_user_input(0, 1, 1, "Price")
writer.set_user_input(0, 1, 2, "120")
writer.set_user_input(0, 2, 1, "Tax")
writer.set_user_input(0, 2, 2, "=B1*0.21")
writer.save_to_xlsx(file)

# Read it back
model = ic.load_from_xlsx(file)
model.evaluate()

# Walk all non-empty cells
(min_row, max_row, min_column, max_column) = model.get_sheet_dimensions(0)
for row in range(min_row, max_row + 1):
    for column in range(min_column, max_column + 1):
        value = model.get_cell_value(0, row, column)
        if value is not None:
            name = ic.column_name_from_number(column)
            print(f"{name}{row}: {value!r}")

# Values come back as native Python types
assert model.get_cell_value(0, 2, 2) == 25.2

# Change an input and recalculate
model.set_user_input(0, 1, 2, "200")
model.evaluate()
assert model.get_cell_value(0, 2, 2) == 42.0
