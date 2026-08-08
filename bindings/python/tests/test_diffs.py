import ironcalc as ic


def test_diffs_sync_two_models():
    model1 = ic.UserModel("model")
    model2 = ic.UserModel("model")

    model1.set_user_input(0, 1, 1, "=1+2")
    model1.set_user_input(0, 1, 2, "=A1+3")
    diffs = model1.flush_send_queue()

    model2.apply_external_diffs(diffs)
    assert model2.get_formatted_cell_value(0, 1, 1) == "3"
    assert model2.get_formatted_cell_value(0, 1, 2) == "6"


def test_diffs_carry_styles():
    model1 = ic.UserModel("model")
    model2 = ic.UserModel("model")

    model1.update_range_style(0, 1, 1, 1, 1, "fill.color", "#ABCDEF")
    model2.apply_external_diffs(model1.flush_send_queue())
    assert model2.get_cell_style(0, 1, 1)["fill"]["color"] == "#ABCDEF"


def test_diffs_carry_sheet_operations():
    model1 = ic.UserModel("model")
    model2 = ic.UserModel("model")

    model1.new_sheet()
    model1.rename_sheet(1, "Data")
    model2.apply_external_diffs(model1.flush_send_queue())
    assert [s["name"] for s in model2.get_worksheets_properties()] == [
        "Sheet1",
        "Data",
    ]


def test_send_queue_is_cleared_after_flush():
    model = ic.UserModel("model")
    model.set_user_input(0, 1, 1, "1")
    first = model.flush_send_queue()
    assert len(first) > 0
    second = model.flush_send_queue()
    assert len(second) == 0 or second != first
