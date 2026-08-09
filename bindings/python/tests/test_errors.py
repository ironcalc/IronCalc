import ironcalc as ic
import pytest


def test_invalid_sheet_raises(um):
    with pytest.raises(ic.WorkbookError):
        um.get_formatted_cell_value(99, 1, 1)


def test_invalid_cell_reference_raises(rm):
    with pytest.raises(ic.WorkbookError):
        rm.get_cell_value_by_ref("NotASheet!A1")


def test_invalid_style_path_raises(um):
    with pytest.raises(ic.WorkbookError):
        um.update_range_style(0, 1, 1, 1, 1, "font.nonsense", "true")


def test_invalid_locale_raises():
    with pytest.raises(ic.WorkbookError):
        ic.create("wb", locale="not-a-locale")


def test_invalid_timezone_raises():
    with pytest.raises(ic.WorkbookError):
        ic.UserModel("wb", tz="Mars/Olympus_Mons")


def test_invalid_color_raises(um):
    with pytest.raises(ic.WorkbookError):
        um.set_sheet_color(0, "not-a-color")


def test_rename_to_existing_sheet_name_raises(um):
    um.new_sheet()
    with pytest.raises(ic.WorkbookError):
        um.rename_sheet(1, "Sheet1")


def test_bad_conditional_formatting_rule_raises(um):
    with pytest.raises(ic.WorkbookError):
        um.add_conditional_formatting(0, "A1:A5", {"type": "NoSuchRule"})
