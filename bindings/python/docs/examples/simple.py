import ironcalc as ic

# The user API: evaluates automatically after every change
model = ic.UserModel("model")
model.set_user_input(0, 1, 1, "=21*2")
assert model.get_formatted_cell_value(0, 1, 1) == "42"

# The raw API: you must evaluate yourself
raw = ic.create("model")
raw.set_user_input(0, 1, 1, "=21*2")
raw.evaluate()
assert raw.get_formatted_cell_value(0, 1, 1) == "42"
