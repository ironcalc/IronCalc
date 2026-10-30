import ironcalc as ic
import pytest


def test_new_defined_name_and_use_in_formula(um):
    um.set_user_input(0, 1, 1, "0.21")
    um.new_defined_name("TaxRate", "Sheet1!$A$1")
    um.set_user_input(0, 2, 1, "=100*TaxRate")
    assert um.get_formatted_cell_value(0, 2, 1) == "21"


def test_defined_name_list(um):
    um.new_defined_name("GlobalName", "Sheet1!$A$1")
    um.new_defined_name("SheetName", "Sheet1!$B$2", scope=0)
    names = um.get_defined_name_list()
    assert {
        "name": "GlobalName",
        "scope": None,
        "formula": "Sheet1!$A$1",
        "hidden": False,
        "comment": "",
    } in names
    assert {
        "name": "SheetName",
        "scope": 0,
        "formula": "Sheet1!$B$2",
        "hidden": False,
        "comment": "",
    } in names


def test_update_defined_name(um):
    um.new_defined_name("OldName", "Sheet1!$A$1")
    um.update_defined_name("OldName", None, "NewName", "Sheet1!$B$1")
    names = um.get_defined_name_list()
    assert len(names) == 1
    assert names[0]["name"] == "NewName"
    assert names[0]["formula"] == "Sheet1!$B$1"


def test_hidden_defined_name(um):
    um.set_user_input(0, 1, 1, "42")
    um.new_defined_name("Secret", "Sheet1!$A$1", hidden=True)
    # a hidden name resolves like any other
    um.set_user_input(0, 2, 1, "=Secret")
    assert um.get_formatted_cell_value(0, 2, 1) == "42"
    assert um.get_defined_name_list()[0]["hidden"] is True
    # update replaces every field: hidden goes back to its default
    um.update_defined_name("Secret", None, "Secret", "Sheet1!$A$1")
    assert um.get_defined_name_list()[0]["hidden"] is False
    um.undo()
    assert um.get_defined_name_list()[0]["hidden"] is True


def test_defined_name_comment(um):
    um.new_defined_name("TaxRate", "Sheet1!$A$1", comment="VAT in 2024")
    assert um.get_defined_name_list()[0]["comment"] == "VAT in 2024"
    um.update_defined_name(
        "TaxRate", None, "TaxRate", "Sheet1!$A$1", new_comment="VAT in 2025"
    )
    assert um.get_defined_name_list()[0]["comment"] == "VAT in 2025"
    um.undo()
    assert um.get_defined_name_list()[0]["comment"] == "VAT in 2024"
    # at most 255 characters
    um.new_defined_name("Ok", "Sheet1!$A$2", comment="x" * 255)
    with pytest.raises(ic.WorkbookError):
        um.new_defined_name("TooLong", "Sheet1!$A$3", comment="x" * 256)


def test_delete_defined_name(um):
    um.new_defined_name("Temp", "Sheet1!$A$1")
    um.delete_defined_name("Temp", None)
    assert um.get_defined_name_list() == []


def test_invalid_defined_name(um):
    with pytest.raises(ic.WorkbookError):
        um.new_defined_name("Not a valid name!", "Sheet1!$A$1")
    with pytest.raises(ic.WorkbookError):
        um.is_valid_defined_name("A1", None, "Sheet1!$B$2")


def test_raw_defined_names(rm):
    rm.new_defined_name("Rate", "Sheet1!$A$1")
    assert rm.get_defined_name_list()[0]["name"] == "Rate"
    assert rm.get_defined_name_list()[0]["hidden"] is False
    rm.new_defined_name("Secret", "Sheet1!$A$2", hidden=True)
    assert rm.get_defined_name_list()[1]["hidden"] is True
    rm.update_defined_name("Secret", None, "Secret", "Sheet1!$A$2", new_comment="note")
    assert rm.get_defined_name_list()[1]["hidden"] is False
    assert rm.get_defined_name_list()[1]["comment"] == "note"
    rm.delete_defined_name("Secret", None)
    rm.delete_defined_name("Rate", None)
    assert rm.get_defined_name_list() == []
