import ironcalc as ic

model = ic.create("model", "en", "UTC")

model.set_user_input(0, 1, 1, "=21*2")
model.evaluate()

assert model.get_formatted_cell_value(0, 1, 1), 42
