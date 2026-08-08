def test_undo_redo_set_input(um):
    assert um.can_undo() is False
    um.set_user_input(0, 1, 1, "42")
    assert um.get_formatted_cell_value(0, 1, 1) == "42"
    assert um.can_undo() is True

    um.undo()
    assert um.get_formatted_cell_value(0, 1, 1) == ""
    assert um.can_redo() is True

    um.redo()
    assert um.get_formatted_cell_value(0, 1, 1) == "42"


def test_undo_style_change(um):
    um.update_range_style(0, 1, 1, 1, 1, "font.b", "true")
    assert um.get_cell_style(0, 1, 1)["font"]["b"] is True
    um.undo()
    # false booleans are omitted from the style dictionary
    assert um.get_cell_style(0, 1, 1)["font"].get("b") is not True


def test_undo_delete_rows_restores_content(um):
    um.set_user_input(0, 1, 1, "important")
    um.delete_rows(0, 1, 1)
    assert um.get_formatted_cell_value(0, 1, 1) == ""
    um.undo()
    assert um.get_formatted_cell_value(0, 1, 1) == "important"


def test_new_edit_clears_redo(um):
    um.set_user_input(0, 1, 1, "1")
    um.undo()
    um.set_user_input(0, 1, 1, "2")
    assert um.can_redo() is False


def test_pause_and_resume_evaluation(um):
    um.set_user_input(0, 1, 1, "1")
    um.pause_evaluation()
    um.set_user_input(0, 1, 2, "=A1+1")
    # not evaluated yet
    assert um.get_formatted_cell_value(0, 1, 2) != "2"
    um.resume_evaluation()
    um.evaluate()
    assert um.get_formatted_cell_value(0, 1, 2) == "2"
