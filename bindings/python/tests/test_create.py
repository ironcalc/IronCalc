import ironcalc as ic

def test_simple():
    model = ic.create("model", "en", "UTC")
    model.set_user_input(0, 1, 1, "=1+2")
    model.evaluate()

    assert model.get_formatted_cell_value(0, 1, 1) == "3"

    bytes = model.to_bytes()

    model2 = ic.load_from_bytes(bytes)
    assert model2.get_formatted_cell_value(0, 1, 1) == "3"


def test_simple_user():
    model = ic.create_user_model("model", "en", "UTC")
    model.set_user_input(0, 1, 1, "=1+2")
    model.set_user_input(0, 1, 2, "=A1+3")

    assert model.get_formatted_cell_value(0, 1, 1) == "3"
    assert model.get_formatted_cell_value(0, 1, 2) == "6"

    diffs = model.flush_send_queue()

    model2 = ic.create_user_model("model", "en", "UTC")
    model2.apply_external_diffs(diffs)
    assert model2.get_formatted_cell_value(0, 1, 1) == "3"
    assert model2.get_formatted_cell_value(0, 1, 2) == "6"
