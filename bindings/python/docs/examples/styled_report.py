"""Build a small styled sales report and save it as xlsx."""

import os
import tempfile

import ironcalc as ic

model = ic.UserModel("sales-report")

sales = [
    ("Apples", 1500, 0.45),
    ("Oranges", 900, 0.61),
    ("Bananas", 2100, 0.32),
]

# Header row
headers = ["Product", "Units", "Price", "Total"]
for column, header in enumerate(headers, start=1):
    model.set_user_input(0, 1, column, header)

# Style the header: bold, white on dark blue, centered
model.update_range_style(0, 1, 1, 1, 4, "font.b", "true")
model.update_range_style(0, 1, 1, 1, 4, "font.color", "#FFFFFF")
model.update_range_style(0, 1, 1, 1, 4, "fill.color", "#2F5597")
model.update_range_style(0, 1, 1, 1, 4, "alignment.horizontal", "center")

# Data rows with a formula for the total
for row, (product, units, price) in enumerate(sales, start=2):
    model.set_user_input(0, row, 1, product)
    model.set_user_input(0, row, 2, str(units))
    model.set_user_input(0, row, 3, str(price))
    model.set_user_input(0, row, 4, f"=B{row}*C{row}")

last_row = 1 + len(sales)

# A grand total below the data
model.set_user_input(0, last_row + 1, 1, "Total")
model.set_user_input(0, last_row + 1, 4, f"=SUM(D2:D{last_row})")
model.update_range_style(0, last_row + 1, 1, last_row + 1, 4, "font.b", "true")

# Currency formats for the price and total columns
model.update_range_style(0, 2, 3, last_row + 1, 4, "num_fmt", "$#,##0.00")

# Borders around the whole table
border = {"item": {"style": "thin", "color": "#2F5597"}, "type": "All"}
model.set_area_with_border(0, 1, 1, last_row + 1, 4, border)

# Wider first column
model.set_columns_width(0, 1, 1, model.get_column_width(0, 1) * 1.5)

assert model.get_formatted_cell_value(0, last_row + 1, 4) == "$1,896.00"

file = os.path.join(tempfile.mkdtemp(), "sales-report.xlsx")
model.save_to_xlsx(file)
print(f"saved {file}")
