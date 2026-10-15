import ironcalc as ic
import pytest


def test_merge_and_get_merged_cells(um):
    um.set_user_input(0, 1, 1, "Title")
    um.merge_cells(0, 1, 1, 2, 3)
    assert um.get_merged_cells(0) == [{"row": 1, "column": 1, "width": 3, "height": 2}]
    assert um.get_cell_content(0, 1, 1) == "Title"


def test_merge_moves_content_to_anchor(um):
    um.set_user_input(0, 2, 2, "Hello")
    um.merge_cells(0, 1, 1, 2, 2)
    assert um.get_cell_content(0, 1, 1) == "Hello"
    assert um.get_cell_content(0, 2, 2) == ""


def test_merge_fails_with_two_content_cells(um):
    um.set_user_input(0, 1, 1, "a")
    um.set_user_input(0, 1, 2, "b")
    with pytest.raises(ic.WorkbookError):
        um.merge_cells(0, 1, 1, 1, 2)
    assert um.get_merged_cells(0) == []


def test_merge_fails_on_single_cell(um):
    with pytest.raises(ic.WorkbookError):
        um.merge_cells(0, 1, 1, 1, 1)


def test_merge_center(um):
    um.set_user_input(0, 1, 1, "Title")
    um.merge_cells_center(0, 1, 1, 1, 3)
    assert um.get_merged_cells(0) == [{"row": 1, "column": 1, "width": 3, "height": 1}]
    assert um.get_cell_style(0, 1, 1)["alignment"]["horizontal"] == "center"


def test_merge_across(um):
    um.merge_cells_across(0, 1, 1, 3, 2)
    assert um.get_merged_cells(0) == [
        {"row": 1, "column": 1, "width": 2, "height": 1},
        {"row": 2, "column": 1, "width": 2, "height": 1},
        {"row": 3, "column": 1, "width": 2, "height": 1},
    ]


def test_merge_down(um):
    um.merge_cells_down(0, 1, 1, 2, 3)
    assert um.get_merged_cells(0) == [
        {"row": 1, "column": 1, "width": 1, "height": 2},
        {"row": 1, "column": 2, "width": 1, "height": 2},
        {"row": 1, "column": 3, "width": 1, "height": 2},
    ]


def test_unmerge(um):
    um.set_user_input(0, 1, 1, "Title")
    um.merge_cells(0, 1, 1, 2, 2)
    um.merge_cells(0, 5, 5, 6, 6)
    # Only touches the first merged cell; unmerging nothing is not an error
    um.unmerge_cells(0, 2, 2, 3, 3)
    assert um.get_merged_cells(0) == [{"row": 5, "column": 5, "width": 2, "height": 2}]
    assert um.get_cell_content(0, 1, 1) == "Title"
    um.unmerge_cells(0, 10, 10, 12, 12)


def test_merge_undo_redo(um):
    um.set_user_input(0, 2, 2, "Hello")
    um.merge_cells(0, 1, 1, 2, 2)
    um.undo()
    assert um.get_merged_cells(0) == []
    assert um.get_cell_content(0, 2, 2) == "Hello"
    um.redo()
    assert um.get_merged_cells(0) == [{"row": 1, "column": 1, "width": 2, "height": 2}]
    assert um.get_cell_content(0, 1, 1) == "Hello"


def test_merged_cells_survive_xlsx_roundtrip(um, tmp_path):
    um.merge_cells(0, 1, 1, 2, 3)
    path = tmp_path / "merged.xlsx"
    um.save_to_xlsx(str(path))
    loaded = ic.UserModel.from_xlsx(str(path))
    assert loaded.get_merged_cells(0) == [{"row": 1, "column": 1, "width": 3, "height": 2}]


def test_raw_model_merge(rm):
    rm.set_user_input(0, 1, 1, "Title")
    rm.merge_cells(0, 1, 1, 1, 2)
    rm.merge_cells_center(0, 2, 1, 2, 2)
    rm.merge_cells_across(0, 3, 1, 4, 2)
    rm.merge_cells_down(0, 5, 1, 6, 2)
    assert len(rm.get_merged_cells(0)) == 6
    rm.unmerge_cells(0, 1, 1, 6, 2)
    assert rm.get_merged_cells(0) == []
    with pytest.raises(ic.WorkbookError):
        rm.get_merged_cells(7)


def test_invalid_sheet(um):
    with pytest.raises(ic.WorkbookError):
        um.get_merged_cells(3)
    with pytest.raises(ic.WorkbookError):
        um.merge_cells(3, 1, 1, 2, 2)
