import ironcalc as ic
import pytest

EXTERNAL = {"type": "External", "target": "https://www.ironcalc.com/", "tooltip": None}
INTERNAL = {"type": "Internal", "location": "Sheet1!A30", "tooltip": "Jump!"}


def test_add_update_delete(um):
    assert um.get_cell_link(0, 2, 2) is None

    um.set_cell_link(0, 2, 2, EXTERNAL)
    assert um.get_cell_link(0, 2, 2) == EXTERNAL

    um.set_cell_link(0, 2, 2, INTERNAL)
    assert um.get_cell_link(0, 2, 2) == INTERNAL

    um.delete_cell_link(0, 2, 2)
    assert um.get_cell_link(0, 2, 2) is None


def test_tooltip_is_optional(um):
    um.set_cell_link(0, 1, 1, {"type": "External", "target": "mailto:daniel@ironcalc.com"})
    link = um.get_cell_link(0, 1, 1)
    assert link == {
        "type": "External",
        "target": "mailto:daniel@ironcalc.com",
        "tooltip": None,
    }


def test_get_links_sorted(um):
    um.set_cell_link(0, 5, 1, EXTERNAL)
    um.set_cell_link(0, 2, 2, INTERNAL)
    links = um.get_links(0)
    assert links == [
        {"row": 2, "column": 2, **INTERNAL},
        {"row": 5, "column": 1, **EXTERNAL},
    ]


def test_undo_redo(um):
    um.set_cell_link(0, 2, 2, EXTERNAL)
    um.undo()
    assert um.get_cell_link(0, 2, 2) is None
    um.redo()
    assert um.get_cell_link(0, 2, 2) == EXTERNAL


def test_invalid_references(um):
    with pytest.raises(ic.WorkbookError):
        um.set_cell_link(1, 1, 1, EXTERNAL)
    with pytest.raises(ic.WorkbookError):
        um.set_cell_link(0, 0, 1, EXTERNAL)


def test_invalid_link_dict(um):
    with pytest.raises(ic.WorkbookError):
        um.set_cell_link(0, 1, 1, {"type": "Nonsense", "target": "x"})


def test_raw_model(rm):
    assert rm.get_cell_link(0, 2, 2) is None
    rm.set_cell_link(0, 2, 2, EXTERNAL)
    assert rm.get_cell_link(0, 2, 2) == EXTERNAL
    assert rm.get_links(0) == [{"row": 2, "column": 2, **EXTERNAL}]
    rm.delete_cell_link(0, 2, 2)
    assert rm.get_cell_link(0, 2, 2) is None
