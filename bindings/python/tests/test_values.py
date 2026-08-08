import ironcalc as ic


def test_raw_model_needs_explicit_evaluate(rm):
    rm.set_user_input(0, 1, 1, "=1+2")
    rm.evaluate()
    assert rm.get_formatted_cell_value(0, 1, 1) == "3"


def test_user_model_evaluates_automatically(um):
    um.set_user_input(0, 1, 1, "=1+2")
    um.set_user_input(0, 1, 2, "=A1+3")
    assert um.get_formatted_cell_value(0, 1, 1) == "3"
    assert um.get_formatted_cell_value(0, 1, 2) == "6"


def test_get_cell_value_returns_native_types(rm):
    rm.set_user_input(0, 1, 1, "42.5")
    rm.set_user_input(0, 2, 1, "Hello")
    rm.set_user_input(0, 3, 1, "TRUE")
    rm.set_user_input(0, 4, 1, "=2*21")
    rm.evaluate()

    assert rm.get_cell_value(0, 1, 1) == 42.5
    assert rm.get_cell_value(0, 2, 1) == "Hello"
    assert rm.get_cell_value(0, 3, 1) is True
    assert rm.get_cell_value(0, 4, 1) == 42.0
    assert rm.get_cell_value(0, 5, 1) is None


def test_get_cell_value_by_ref(rm):
    rm.set_user_input(0, 4, 3, "3.25")
    rm.evaluate()
    assert rm.get_cell_value_by_ref("Sheet1!C4") == 3.25


def test_cell_types(um):
    um.set_user_input(0, 1, 1, "42")
    um.set_user_input(0, 2, 1, "Hello")
    um.set_user_input(0, 3, 1, "TRUE")
    um.set_user_input(0, 4, 1, "=1/0")

    assert um.get_cell_type(0, 1, 1) == ic.CellType.Number
    assert um.get_cell_type(0, 2, 1) == ic.CellType.Text
    assert um.get_cell_type(0, 3, 1) == ic.CellType.LogicalValue
    assert um.get_cell_type(0, 4, 1) == ic.CellType.ErrorValue


def test_get_cell_content_returns_formula(um):
    um.set_user_input(0, 1, 1, "=1+2")
    um.set_user_input(0, 2, 1, "plain text")
    assert um.get_cell_content(0, 1, 1) == "=1+2"
    assert um.get_cell_content(0, 2, 1) == "plain text"


def test_update_cell_without_parsing(rm):
    # update_cell_with_text does not try to parse the input
    rm.update_cell_with_text(0, 1, 1, "123")
    rm.update_cell_with_number(0, 2, 1, 1.5)
    rm.update_cell_with_bool(0, 3, 1, False)
    rm.update_cell_with_formula(0, 4, 1, "=A2*2")
    rm.evaluate()

    assert rm.get_cell_value(0, 1, 1) == "123"
    assert rm.get_cell_type(0, 1, 1) == ic.CellType.Text
    assert rm.get_cell_value(0, 2, 1) == 1.5
    assert rm.get_cell_value(0, 3, 1) is False
    assert rm.get_cell_formula(0, 4, 1) == "=A2*2"
    assert rm.get_cell_value(0, 4, 1) == 3.0


def test_is_empty_cell(rm):
    rm.set_user_input(0, 1, 1, "something")
    assert rm.is_empty_cell(0, 1, 1) is False
    assert rm.is_empty_cell(0, 5, 5) is True


def test_get_all_cells(rm):
    rm.set_user_input(0, 1, 1, "a")
    rm.set_user_input(0, 3, 2, "b")
    assert set(rm.get_all_cells()) == {(0, 1, 1), (0, 3, 2)}


def test_formatted_value_currency(um):
    um.update_range_style(0, 1, 1, 1, 1, "num_fmt", "$#,##0.00")
    um.set_user_input(0, 1, 1, "1234.5")
    assert um.get_formatted_cell_value(0, 1, 1) == "$1,234.50"


def test_array_formula_spills(um):
    um.set_user_input(0, 1, 1, "1")
    um.set_user_input(0, 2, 1, "2")
    um.set_user_input(0, 3, 1, "3")
    um.set_user_array_formula(0, 1, 2, 1, 3, "=A1:A3*10")
    assert um.get_formatted_cell_value(0, 1, 2) == "10"
    assert um.get_formatted_cell_value(0, 2, 2) == "20"
    assert um.get_formatted_cell_value(0, 3, 2) == "30"


def test_sheet_dimensions(rm):
    assert rm.get_sheet_dimensions(0) == (1, 1, 1, 1)
    rm.set_user_input(0, 3, 5, "Hello")
    rm.set_user_input(0, 10, 8, "World")
    rm.evaluate()
    assert rm.get_sheet_dimensions(0) == (3, 10, 5, 8)


def test_sheet_markup(rm):
    # the markup shows formulas, not values
    rm.set_user_input(0, 1, 1, "=1+1")
    rm.evaluate()
    assert "=1+1" in rm.get_sheet_markup(0)
