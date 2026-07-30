import ironcalc as ic
import pytest


def test_new_defined_name_and_use_in_formula(um):
    um.set_user_input(0, 1, 1, "0.21")
    um.new_defined_name("TaxRate", None, "Sheet1!$A$1")
    um.set_user_input(0, 2, 1, "=100*TaxRate")
    assert um.get_formatted_cell_value(0, 2, 1) == "21"


def test_defined_name_list(um):
    um.new_defined_name("GlobalName", None, "Sheet1!$A$1")
    um.new_defined_name("SheetName", 0, "Sheet1!$B$2")
    names = um.get_defined_name_list()
    assert {"name": "GlobalName", "scope": None, "formula": "Sheet1!$A$1"} in names
    assert {"name": "SheetName", "scope": 0, "formula": "Sheet1!$B$2"} in names


def test_update_defined_name(um):
    um.new_defined_name("OldName", None, "Sheet1!$A$1")
    um.update_defined_name("OldName", None, "NewName", None, "Sheet1!$B$1")
    names = um.get_defined_name_list()
    assert len(names) == 1
    assert names[0]["name"] == "NewName"
    assert names[0]["formula"] == "Sheet1!$B$1"


def test_delete_defined_name(um):
    um.new_defined_name("Temp", None, "Sheet1!$A$1")
    um.delete_defined_name("Temp", None)
    assert um.get_defined_name_list() == []


def test_invalid_defined_name(um):
    with pytest.raises(ic.WorkbookError):
        um.new_defined_name("Not a valid name!", None, "Sheet1!$A$1")
    with pytest.raises(ic.WorkbookError):
        um.is_valid_defined_name("A1", None, "Sheet1!$B$2")


def test_raw_defined_names(rm):
    rm.new_defined_name("Rate", None, "Sheet1!$A$1")
    assert rm.get_defined_name_list()[0]["name"] == "Rate"
    rm.delete_defined_name("Rate", None)
    assert rm.get_defined_name_list() == []
