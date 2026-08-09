import ironcalc as ic
import pytest


def sheet_names(model):
    return [s["name"] for s in model.get_worksheets_properties()]


def test_new_and_rename_sheet(um):
    um.new_sheet()
    assert len(sheet_names(um)) == 2
    um.rename_sheet(1, "Data")
    assert sheet_names(um) == ["Sheet1", "Data"]


def test_delete_sheet(um):
    um.new_sheet()
    um.delete_sheet(0)
    assert len(sheet_names(um)) == 1


def test_cannot_delete_only_sheet(um):
    with pytest.raises(ic.WorkbookError):
        um.delete_sheet(0)


def test_duplicate_sheet(um):
    um.set_user_input(0, 1, 1, "42")
    um.duplicate_sheet(0)
    assert len(sheet_names(um)) == 2
    assert um.get_formatted_cell_value(1, 1, 1) == "42"


def test_hide_and_unhide_sheet(um):
    um.new_sheet()
    um.hide_sheet(1)
    assert um.get_worksheets_properties()[1]["state"] == "hidden"
    um.unhide_sheet(1)
    assert um.get_worksheets_properties()[1]["state"] == "visible"


def test_move_sheet(um):
    um.new_sheet()
    um.rename_sheet(1, "Second")
    um.move_sheet(1, 0)
    assert sheet_names(um) == ["Second", "Sheet1"]


def test_sheet_color(um):
    um.set_sheet_color(0, "#FF5566")
    assert um.get_worksheets_properties()[0]["color"] == "#FF5566"
    um.set_sheet_color(0, None)
    # the color key is absent when the sheet has no color
    assert um.get_worksheets_properties()[0].get("color") is None


def test_cross_sheet_formula(um):
    um.new_sheet()
    um.rename_sheet(1, "Data")
    um.set_user_input(1, 1, 1, "21")
    um.set_user_input(0, 1, 1, "=Data!A1*2")
    assert um.get_formatted_cell_value(0, 1, 1) == "42"


def test_show_grid_lines(um):
    assert um.get_show_grid_lines(0) is True
    um.set_show_grid_lines(0, False)
    assert um.get_show_grid_lines(0) is False


def test_raw_add_sheet_and_state(rm):
    rm.add_sheet("MySheet")
    assert sheet_names(rm) == ["Sheet1", "MySheet"]
    rm.set_sheet_state(1, "hidden")
    assert rm.get_worksheets_properties()[1]["state"] == "hidden"
    rm.set_sheet_state(1, "visible")
    assert rm.get_worksheets_properties()[1]["state"] == "visible"
    with pytest.raises(ic.WorkbookError):
        rm.set_sheet_state(1, "invisible")
