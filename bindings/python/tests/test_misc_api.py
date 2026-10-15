import ironcalc as ic
import pytest


def test_cycle_reference(um):
    assert um.cycle_reference("=A1", 3, 3) == ("=$A$1", 5, 5)
    assert um.cycle_reference("=$A$1", 5, 5) == ("=A$1", 4, 4)
    assert um.cycle_reference("=A$1", 4, 4) == ("=$A1", 4, 4)
    assert um.cycle_reference("=$A1", 4, 4) == ("=A1", 3, 3)


def test_cycle_reference_raw(rm):
    assert rm.cycle_reference("=A1", 3, 3) == ("=$A$1", 5, 5)


def test_to_xlsx_bytes(um, tmp_path):
    um.set_user_input(0, 1, 1, "=1+2")
    data = um.to_xlsx()
    assert isinstance(data, bytes)
    assert data[:2] == b"PK"
    path = tmp_path / "book.xlsx"
    path.write_bytes(data)
    loaded = ic.UserModel.from_xlsx(str(path))
    assert loaded.get_formatted_cell_value(0, 1, 1) == "3"


def test_to_xlsx_bytes_raw(rm, tmp_path):
    rm.set_user_input(0, 1, 1, "Hello")
    path = tmp_path / "book.xlsx"
    path.write_bytes(rm.to_xlsx())
    assert ic.load_from_xlsx(str(path)).get_cell_content(0, 1, 1) == "Hello"


def test_rows_and_columns_with_data(um):
    um.set_user_input(0, 2, 3, "a")
    um.set_user_input(0, 5, 3, "b")
    um.set_user_input(0, 5, 7, "c")
    assert um.get_rows_with_data(0, 3) == [2, 5]
    assert um.get_rows_with_data(0, 4) == []
    assert um.get_columns_with_data(0, 5) == [3, 7]
    assert um.get_columns_with_data(0, 1) == []
    with pytest.raises(ic.WorkbookError):
        um.get_rows_with_data(3, 1)


def test_get_theme_list(um):
    themes = ic.get_theme_list()
    assert len(themes) > 1
    assert all("name" in t for t in themes)
    # Themes from the list can be applied to a workbook
    um.set_theme(themes[-1])
    assert um.get_theme()["name"] == themes[-1]["name"]


def test_hex_with_tint_to_rgb():
    assert ic.hex_with_tint_to_rgb("#000000", 0.0) == "#000000"
    assert ic.hex_with_tint_to_rgb("#FFFFFF", -1.0) == "#000000"
    assert ic.hex_with_tint_to_rgb("#000000", 1.0) == "#FFFFFF"
