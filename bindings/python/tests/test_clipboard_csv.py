def test_copy_paste_adjusts_formulas(um):
    um.set_user_input(0, 1, 1, "1")
    um.set_user_input(0, 2, 1, "2")
    um.set_user_input(0, 3, 1, "=A1+A2")

    um.set_selected_range(1, 1, 3, 1)
    clipboard = um.copy_to_clipboard()
    assert "1" in clipboard["csv"]

    # paste in column C
    um.set_selected_cell(1, 3)
    um.paste_from_clipboard(
        clipboard["sheet"], tuple(clipboard["range"]), clipboard["data"], False
    )
    assert um.get_formatted_cell_value(0, 1, 3) == "1"
    assert um.get_formatted_cell_value(0, 2, 3) == "2"
    # relative references were adjusted
    assert um.get_cell_content(0, 3, 3) == "=C1+C2"
    assert um.get_formatted_cell_value(0, 3, 3) == "3"


def test_cut_paste_removes_source(um):
    um.set_user_input(0, 1, 1, "moved")
    um.set_selected_range(1, 1, 1, 1)
    clipboard = um.copy_to_clipboard()

    um.set_selected_cell(5, 5)
    um.paste_from_clipboard(
        clipboard["sheet"], tuple(clipboard["range"]), clipboard["data"], True
    )
    assert um.get_formatted_cell_value(0, 5, 5) == "moved"
    assert um.get_formatted_cell_value(0, 1, 1) == ""


def test_paste_csv(um):
    csv = "name\tvalue\napples\t3\noranges\t5"
    um.paste_csv_string(0, 1, 1, 3, 2, csv)
    assert um.get_formatted_cell_value(0, 1, 1) == "name"
    assert um.get_formatted_cell_value(0, 2, 1) == "apples"
    assert um.get_formatted_cell_value(0, 2, 2) == "3"
    assert um.get_formatted_cell_value(0, 3, 2) == "5"
