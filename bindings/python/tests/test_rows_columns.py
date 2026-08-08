def test_insert_rows_adjusts_formulas(um):
    um.set_user_input(0, 1, 1, "10")
    um.set_user_input(0, 2, 1, "=A1*2")
    um.insert_rows(0, 1, 2)
    # the data moved down two rows
    assert um.get_formatted_cell_value(0, 3, 1) == "10"
    assert um.get_cell_content(0, 4, 1) == "=A3*2"
    assert um.get_formatted_cell_value(0, 4, 1) == "20"


def test_delete_rows(um):
    um.set_user_input(0, 1, 1, "first")
    um.set_user_input(0, 5, 1, "fifth")
    um.delete_rows(0, 1, 4)
    assert um.get_formatted_cell_value(0, 1, 1) == "fifth"


def test_insert_and_delete_columns(um):
    um.set_user_input(0, 1, 1, "A")
    um.set_user_input(0, 1, 2, "B")
    um.insert_columns(0, 2, 1)
    assert um.get_formatted_cell_value(0, 1, 2) == ""
    assert um.get_formatted_cell_value(0, 1, 3) == "B"
    um.delete_columns(0, 2, 1)
    assert um.get_formatted_cell_value(0, 1, 2) == "B"


def test_move_columns(um):
    um.set_user_input(0, 1, 1, "one")
    um.set_user_input(0, 1, 2, "two")
    um.move_columns(0, 1, 1, 1)
    assert um.get_formatted_cell_value(0, 1, 1) == "two"
    assert um.get_formatted_cell_value(0, 1, 2) == "one"


def test_move_rows(um):
    um.set_user_input(0, 1, 1, "one")
    um.set_user_input(0, 2, 1, "two")
    um.move_rows(0, 1, 1, 1)
    assert um.get_formatted_cell_value(0, 1, 1) == "two"
    assert um.get_formatted_cell_value(0, 2, 1) == "one"


def test_column_widths_and_row_heights(um):
    default_width = um.get_column_width(0, 1)
    um.set_columns_width(0, 1, 3, default_width * 2)
    assert um.get_column_width(0, 2) == default_width * 2
    assert um.get_column_width(0, 4) == default_width

    default_height = um.get_row_height(0, 1)
    um.set_rows_height(0, 1, 2, default_height + 10)
    assert um.get_row_height(0, 1) == default_height + 10
    assert um.get_row_height(0, 3) == default_height


def test_raw_column_width_row_height(rm):
    rm.set_column_width(0, 1, 100.0)
    assert rm.get_column_width(0, 1) == 100.0
    rm.set_row_height(0, 1, 42.0)
    assert rm.get_row_height(0, 1) == 42.0


def test_hidden_rows_and_columns(um):
    um.set_rows_hidden(0, 2, 3, True)
    um.set_columns_hidden(0, 2, 3, True)
    # the user model has no is_hidden getter; check through the raw model
    import ironcalc as ic

    raw = ic.load_from_bytes(bytes(um.to_bytes()))
    assert raw.is_row_hidden(0, 2) is True
    assert raw.is_row_hidden(0, 1) is False
    assert raw.is_column_hidden(0, 3) is True
    assert raw.is_column_hidden(0, 4) is False


def test_raw_hidden(rm):
    rm.set_row_hidden(0, 1, True)
    rm.set_column_hidden(0, 2, True)
    assert rm.is_row_hidden(0, 1) is True
    assert rm.is_column_hidden(0, 2) is True


def test_frozen_rows_and_columns(um):
    assert um.get_frozen_rows_count(0) == 0
    um.set_frozen_rows_count(0, 2)
    um.set_frozen_columns_count(0, 1)
    assert um.get_frozen_rows_count(0) == 2
    assert um.get_frozen_columns_count(0) == 1


def test_non_empty_helpers(um):
    um.set_user_input(0, 1, 2, "a")
    um.set_user_input(0, 1, 5, "b")
    assert um.get_last_non_empty_in_row_before_column(0, 1, 4) == 2
    assert um.get_first_non_empty_in_row_after_column(0, 1, 3) == 5
    assert um.get_last_non_empty_in_row_before_column(0, 2, 4) is None
