"""Highlight interesting values with conditional formatting."""

import os
import tempfile
import random

import ironcalc as ic

model = ic.UserModel("cf-example")

random.seed(42)
for row in range(1, 21):
    model.set_user_input(0, row, 1, str(random.randint(0, 100)))
    model.set_user_input(0, row, 2, str(random.randint(0, 100)))

# A color scale over the first column: red (low) to green (high)
model.add_conditional_formatting(
    0,
    "A1:A20",
    {
        "type": "ColorScale",
        "thresholds": [
            {"cfvo": "Min", "color": "#F8696B"},
            {"cfvo": {"Percentile": 50}, "color": "#FFEB84"},
            {"cfvo": "Max", "color": "#63BE7B"},
        ],
    },
)

# Highlight values greater than 90 in the second column
model.add_conditional_formatting(
    0,
    "B1:B20",
    {
        "type": "CellIs",
        "operator": "GreaterThan",
        "formula": "90",
        "formula2": None,
        "format": {"fill": {"color": "#FFC7CE"}, "font": {"b": True}},
        "stop_if_true": False,
    },
)

rules = model.get_conditional_formatting_list(0)
assert len(rules) == 2

file = os.path.join(tempfile.mkdtemp(), "conditional-formatting.xlsx")
model.save_to_xlsx(file)
print(f"saved {file}")
