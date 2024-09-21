import ironcalc as ic

def test_simple():
    model = ic.create("model", "en", "UTC")
    model.set_user_input(0, 1, 1, "=1+2")
    model.evaluate()

    assert model.get_formatted_cell_value(0, 1, 1) == "3"
