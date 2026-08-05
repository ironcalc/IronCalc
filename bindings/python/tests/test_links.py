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
    um.set_cell_link(0, 1, 1, {"type": "External", "target": "mailto:hello@ironcalc.com"})
    link = um.get_cell_link(0, 1, 1)
    assert link == {
        "type": "External",
        "target": "mailto:hello@ironcalc.com",
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


def test_label_and_style_in_a_single_undo_step(um):
    um.set_cell_link(0, 2, 2, EXTERNAL, "IronCalc")
    # the label is the cell content and the link style is applied
    assert um.get_formatted_cell_value(0, 2, 2) == "IronCalc"
    style = um.get_cell_style(0, 2, 2)
    assert style["font"].get("u", False) is True
    # one undo reverts the link, the content and the style together
    um.undo()
    assert um.get_cell_link(0, 2, 2) is None
    assert um.get_formatted_cell_value(0, 2, 2) == ""
    assert um.get_cell_style(0, 2, 2)["font"].get("u", False) is False
    assert not um.can_undo()
    # one redo restores everything
    um.redo()
    assert um.get_cell_link(0, 2, 2) == EXTERNAL
    assert um.get_formatted_cell_value(0, 2, 2) == "IronCalc"
    assert um.get_cell_style(0, 2, 2)["font"].get("u", False) is True


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
