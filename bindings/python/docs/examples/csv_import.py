"""Import tab separated data and add formulas over it."""

import ironcalc as ic

model = ic.UserModel("csv-import")

rows = [
    "month\trevenue\tcosts",
    "January\t10000\t8000",
    "February\t12000\t8500",
    "March\t14000\t9000",
]
model.paste_csv_string(0, 1, 1, len(rows), 3, "\n".join(rows))

# Add a profit column
model.set_user_input(0, 1, 4, "profit")
for row in range(2, len(rows) + 1):
    model.set_user_input(0, row, 4, f"=B{row}-C{row}")

assert model.get_formatted_cell_value(0, 2, 4) == "2000"
assert model.get_formatted_cell_value(0, 4, 4) == "5000"
print(model.get_formatted_cell_value(0, 4, 4))
