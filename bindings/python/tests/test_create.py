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


def test_sheet_dimensions():
    # Test with empty sheet
    model = ic.create("model", "en", "UTC")
    min_row, max_row, min_col, max_col = model.get_sheet_dimensions(0)
    assert (min_row, max_row, min_col, max_col) == (1, 1, 1, 1)
    
    # Add some cells
    model.set_user_input(0, 3, 5, "Hello")
    model.set_user_input(0, 10, 8, "World")
    model.evaluate()
    
    # Check dimensions - should span from (3,5) to (10,8)
    min_row, max_row, min_col, max_col = model.get_sheet_dimensions(0)
    assert (min_row, max_row, min_col, max_col) == (3, 10, 5, 8)


def test_sheet_dimensions_user_model():
    # Test with user model API as well
    model = ic.create_user_model("model", "en", "UTC")
    
    # Add a single cell
    model.set_user_input(0, 2, 3, "Test")
    
    # Check dimensions
    min_row, max_row, min_col, max_col = model.get_sheet_dimensions(0)
    assert (min_row, max_row, min_col, max_col) == (2, 2, 3, 3)
